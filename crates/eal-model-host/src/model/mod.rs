//! Revision-bound native Qwen2 inference. All artifact reads use safe Rust APIs.
use anyhow::{Context, Result, bail, ensure};
use candle_core::{DType, Device, Shape, Tensor};
use candle_nn::var_builder::SimpleBackend;
use candle_nn::{Init, VarBuilder};
use candle_transformers::models::qwen2::{Config, ModelForCausalLM};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{collections::HashMap, fs, path::Path, time::Instant};
use tokenizers::Tokenizer;

use crate::contract::{Generation, Generator};

pub use crate::bundle::sha256;

pub const BASE_MODEL: &str = "Qwen/Qwen2.5-0.5B-Instruct";
pub const BASE_REVISION: &str = "7ae557604adf67be50417f59c2c2f167def9a775";
pub const BASE_CONFIG_SHA256: &str =
    "18e18afcaccafade98daf13a54092927904649e1dd4eba8299ab717d5d94ff45";
pub const BASE_GENERATION_CONFIG_SHA256: &str =
    "e558847a8b4402616f1273797b015104dc266fe4b520056fca88823ba8f8ebe6";
pub const BASE_WEIGHTS_SHA256: &str =
    "fdf756fa7fcbe7404d5c60e26bff1a0c8b8aa1f72ced49e7dd0210fe288fb7fe";

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct LoadOptions {
    pub precision: String,
    pub device: String,
    pub prefix_cache: bool,
}
impl Default for LoadOptions {
    fn default() -> Self {
        Self {
            precision: "f32".into(),
            device: "cpu".into(),
            prefix_cache: true,
        }
    }
}

pub struct ChatTokenizer {
    tokenizer: Tokenizer,
    template: String,
    eos_token: u32,
}
impl ChatTokenizer {
    pub fn load(run_dir: &Path) -> Result<Self> {
        let dir = run_dir.join("tokenizer");
        let tokenizer = Tokenizer::from_file(dir.join("tokenizer.json"))
            .map_err(|e| anyhow::anyhow!("load tokenizer: {e}"))?;
        let config: Value = serde_json::from_slice(&fs::read(dir.join("tokenizer_config.json"))?)?;
        let eos_name = config["eos_token"].as_str().context("missing eos token")?;
        let eos_token = tokenizer
            .token_to_id(eos_name)
            .context("eos token absent from vocabulary")?;
        ensure!(eos_token == 151645, "unexpected pinned EOS token");
        Ok(Self {
            tokenizer,
            template: fs::read_to_string(dir.join("chat_template.jinja"))?,
            eos_token,
        })
    }
    pub fn render(&self, messages: &[Value], add_generation_prompt: bool) -> Result<String> {
        ensure!(!messages.is_empty(), "chat messages must not be empty");
        for message in messages {
            ensure!(
                message["role"].is_string() && message["content"].is_string(),
                "each message requires string role and content"
            );
        }
        let mut env = minijinja::Environment::new();
        env.add_template("chat", &self.template)?;
        Ok(env.get_template("chat")?.render(minijinja::context! {
            messages => messages, add_generation_prompt => add_generation_prompt,
            eos_token => "<|im_end|>", bos_token => "<|endoftext|>", tools => false,
        })?)
    }
    pub fn encode_chat(&self, messages: &[Value], add_generation_prompt: bool) -> Result<Vec<u32>> {
        self.encode(&self.render(messages, add_generation_prompt)?)
    }
    pub fn encode(&self, text: &str) -> Result<Vec<u32>> {
        Ok(self
            .tokenizer
            .encode(text, false)
            .map_err(|e| anyhow::anyhow!("tokenize: {e}"))?
            .get_ids()
            .to_vec())
    }
    pub fn decode(&self, ids: &[u32]) -> Result<String> {
        self.tokenizer
            .decode(ids, true)
            .map_err(|e| anyhow::anyhow!("decode: {e}"))
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct LoraConfig {
    pub r: usize,
    pub lora_alpha: f64,
    pub lora_dropout: f64,
    pub target_modules: Vec<String>,
    pub fan_in_fan_out: bool,
    pub bias: String,
    #[serde(default)]
    pub use_dora: bool,
    #[serde(default)]
    pub use_rslora: bool,
}
impl LoraConfig {
    pub fn load(run_dir: &Path) -> Result<Self> {
        let raw: Value =
            serde_json::from_slice(&fs::read(run_dir.join("adapter/adapter_config.json"))?)?;
        ensure!(
            raw["peft_type"] == "LORA" && raw["task_type"] == "CAUSAL_LM",
            "unsupported adapter type"
        );
        ensure!(
            raw["revision"] == BASE_REVISION && raw["base_model_name_or_path"] == BASE_MODEL,
            "adapter base identity differs from pinned model"
        );
        for key in ["rank_pattern", "alpha_pattern"] {
            ensure!(
                raw[key].as_object().is_some_and(|v| v.is_empty()),
                "per-layer {key} is not supported"
            );
        }
        for key in [
            "layer_replication",
            "layers_to_transform",
            "layers_pattern",
            "target_parameters",
            "modules_to_save",
        ] {
            ensure!(raw[key].is_null(), "adapter option {key} is not supported");
        }
        ensure!(
            raw["use_qalora"] != true && raw["lora_bias"] != true,
            "unsupported adapter quantisation or bias variant"
        );
        let cfg: Self = serde_json::from_value(raw)?;
        ensure!(
            cfg.r > 0 && cfg.lora_alpha.is_finite(),
            "invalid LoRA rank or alpha"
        );
        ensure!(
            !cfg.fan_in_fan_out && cfg.bias == "none" && !cfg.use_dora && !cfg.use_rslora,
            "unsupported PEFT LoRA variant"
        );
        ensure!(
            cfg.target_modules.len() == 2
                && cfg.target_modules.iter().any(|v| v == "q_proj")
                && cfg.target_modules.iter().any(|v| v == "v_proj"),
            "only pinned q_proj/v_proj LoRA target modules are supported"
        );
        Ok(cfg)
    }
    pub fn scale(&self) -> f64 {
        self.lora_alpha / self.r as f64
    }
}

/// Lazy safe-buffer loader merges each low-rank projection once during model construction.
struct MergedBackend {
    base: candle_core::safetensors::BufferedSafetensors,
    adapters: HashMap<String, Tensor>,
    scale: f64,
}
impl SimpleBackend for MergedBackend {
    fn get(
        &self,
        shape: Shape,
        name: &str,
        _: Init,
        dtype: DType,
        dev: &Device,
    ) -> candle_core::Result<Tensor> {
        let mut weight = self.base.load(name, dev)?.to_dtype(DType::F32)?;
        if let Some(stem) = name.strip_suffix(".weight") {
            let a_name = format!("base_model.model.{stem}.lora_A.weight");
            let b_name = format!("base_model.model.{stem}.lora_B.weight");
            if let Some(a) = self.adapters.get(&a_name) {
                let b = self
                    .adapters
                    .get(&b_name)
                    .ok_or_else(|| candle_core::Error::Msg(format!("missing {b_name}")))?;
                weight = (&weight + (b.matmul(a)? * self.scale)?)?;
            }
        }
        if weight.shape() != &shape {
            candle_core::bail!(
                "shape mismatch for {name}: {:?} != {:?}",
                weight.shape(),
                shape
            )
        }
        weight.to_dtype(dtype)
    }
    fn contains_tensor(&self, name: &str) -> bool {
        self.base.get(name).is_ok()
    }
}

struct PrefixSnapshot {
    tokens: Vec<u32>,
    model: ModelForCausalLM,
}
#[derive(Default)]
struct CacheStats {
    hit: bool,
    tokens: usize,
    prefix_forward_seconds: f64,
}
/// The cache is limited to an independently rendered leading system message.
fn eligible_system_prefix(ids: &[u32], prefix: &[u32]) -> bool {
    !prefix.is_empty()
        && prefix.len() <= 1024
        && prefix.len() < ids.len()
        && ids.starts_with(prefix)
}

pub struct QwenGenerator {
    model: ModelForCausalLM,
    tokenizer: ChatTokenizer,
    device: Device,
    options: LoadOptions,
    identity: Value,
    bundle_verification: Value,
    adapter_hash: String,
    pub load_seconds: f64,
    context_limit: usize,
    prefix_snapshot: Option<PrefixSnapshot>,
    repetition_penalty: f32,
}
impl QwenGenerator {
    pub fn load(base_dir: &Path, run_dir: &Path, options: LoadOptions) -> Result<Self> {
        let start = Instant::now();
        // This fixed-bundle verifier is the authority for all weights, configuration
        // and tokenizer assets. It runs before any tokenizer or tensor is read.
        let bundle_verification = crate::bundle::verify_pinned_bundle(base_dir, run_dir)?;
        let identity: Value =
            serde_json::from_slice(&fs::read(run_dir.join("run_identity.json"))?)?;
        ensure!(
            options.precision == "f32",
            "the pinned inference host supports f32 precision"
        );
        let base_path = base_dir.join("model.safetensors");
        let generation_config: Value =
            serde_json::from_slice(&fs::read(base_dir.join("generation_config.json"))?)?;
        let repetition_penalty = generation_config["repetition_penalty"]
            .as_f64()
            .context("pinned generation configuration has no repetition penalty")?
            as f32;
        ensure!(
            repetition_penalty == 1.1f32,
            "the pinned generation penalty must be 1.1"
        );
        let config: Config = serde_json::from_slice(&fs::read(base_dir.join("config.json"))?)?;
        ensure!(
            config.hidden_size == 896
                && config.num_hidden_layers == 24
                && config.num_attention_heads == 14
                && config.num_key_value_heads == 2
                && config.vocab_size == 151936
                && config.tie_word_embeddings,
            "base model configuration differs from pinned architecture"
        );
        let dtype = DType::F32;
        let device = select_device(&options.device)?;
        let tokenizer = ChatTokenizer::load(run_dir)?;
        let lora = LoraConfig::load(run_dir)?;
        let adapter_hash = sha256(&run_dir.join("adapter/adapter_model.safetensors"))?;
        let adapters = candle_core::safetensors::load(
            run_dir.join("adapter/adapter_model.safetensors"),
            &device,
        )?
        .into_iter()
        .map(|(k, v)| Ok((k, v.to_dtype(DType::F32)?)))
        .collect::<candle_core::Result<HashMap<_, _>>>()?;
        validate_adapter_names(&adapters, &config, lora.r)?;
        let backend = MergedBackend {
            base: candle_core::safetensors::BufferedSafetensors::new(fs::read(&base_path)?)?,
            adapters,
            scale: lora.scale(),
        };
        let vb = VarBuilder::from_backend(Box::new(backend), dtype, device.clone());
        let model = ModelForCausalLM::new(&config, vb)?;
        Ok(Self {
            model,
            tokenizer,
            device,
            options,
            identity,
            bundle_verification,
            adapter_hash,
            load_seconds: start.elapsed().as_secs_f64(),
            context_limit: config.max_position_embeddings.min(4096),
            prefix_snapshot: None,
            repetition_penalty,
        })
    }
    pub fn bundle_identity(&self) -> &Value {
        &self.bundle_verification
    }
    pub fn effective_repetition_penalty(&self) -> f32 {
        self.repetition_penalty
    }
    /// Raw-token callers have no trusted system-message boundary, so no prefix is retained.
    pub fn generate_tokens(
        &mut self,
        ids: &[u32],
        max_new_tokens: usize,
    ) -> Result<(Vec<u32>, f64, f64)> {
        let (tokens, prefill, decode, _) =
            self.generate_tokens_with_prefix(ids, max_new_tokens, None)?;
        Ok((tokens, prefill, decode))
    }
    fn generate_tokens_with_prefix(
        &mut self,
        ids: &[u32],
        max_new_tokens: usize,
        prefix: Option<&[u32]>,
    ) -> Result<(Vec<u32>, f64, f64, CacheStats)> {
        self.model.clear_kv_cache();
        let result = self.generate_tokens_with_prefix_inner(ids, max_new_tokens, prefix);
        // Only the separately retained system-prefix snapshot can survive a call.
        self.model.clear_kv_cache();
        result
    }
    fn generate_tokens_with_prefix_inner(
        &mut self,
        ids: &[u32],
        max_new_tokens: usize,
        prefix: Option<&[u32]>,
    ) -> Result<(Vec<u32>, f64, f64, CacheStats)> {
        ensure!(
            (1..=512).contains(&max_new_tokens),
            "max_new_tokens must be between 1 and 512"
        );
        ensure!(!ids.is_empty(), "prompt is empty");
        let limit = self.context_limit.min(4096);
        ensure!(
            ids.len()
                .checked_add(max_new_tokens)
                .is_some_and(|n| n <= limit),
            "prompt plus requested generation exceeds the host context limit ({limit})"
        );
        let mut cache = CacheStats::default();
        let mut offset = 0;
        let mut prefill_seconds = 0.;
        let prefix = prefix
            .filter(|prefix| self.options.prefix_cache && eligible_system_prefix(ids, prefix));
        if let Some(prefix) = prefix {
            let started = Instant::now();
            cache.tokens = prefix.len();
            if let Some(snapshot) = self
                .prefix_snapshot
                .as_ref()
                .filter(|snapshot| snapshot.tokens == prefix)
            {
                self.model = snapshot.model.clone();
                cache.hit = true;
            } else {
                // Drop a previous system snapshot before retaining its replacement.
                self.prefix_snapshot = None;
                let prefix_input = Tensor::new(prefix, &self.device)?.unsqueeze(0)?;
                let logits = self.model.forward(&prefix_input, 0)?;
                drop(logits);
                cache.prefix_forward_seconds = started.elapsed().as_secs_f64();
                let snapshot = self.model.clone();
                self.prefix_snapshot = Some(PrefixSnapshot {
                    tokens: prefix.to_vec(),
                    model: snapshot,
                });
            }
            offset = prefix.len();
            prefill_seconds = started.elapsed().as_secs_f64();
        } else {
            // A missing, changed-type, long or nonmatching prefix must not reuse stored state.
            self.prefix_snapshot = None;
        }
        let mut output = Vec::new();
        let mut input = ids[offset..].to_vec();
        let mut decode_seconds = 0.;
        // HF applies this processor to the full prompt plus every generated token.
        let mut seen_context = ids.to_vec();
        for step in 0..max_new_tokens {
            let t = Instant::now();
            let tensor = Tensor::new(input.as_slice(), &self.device)?.unsqueeze(0)?;
            let logits = self
                .model
                .forward(&tensor, offset)?
                .flatten_all()?
                .to_dtype(DType::F32)?;
            let logits = if self.repetition_penalty == 1. {
                logits
            } else {
                candle_transformers::utils::apply_repeat_penalty(
                    &logits,
                    self.repetition_penalty,
                    &seen_context,
                )?
            };
            let token = logits.argmax(0)?.to_scalar::<u32>()?;
            let elapsed = t.elapsed().as_secs_f64();
            if step == 0 {
                prefill_seconds += elapsed;
            } else {
                decode_seconds += elapsed;
            }
            offset += input.len();
            output.push(token);
            seen_context.push(token);
            if token == self.tokenizer.eos_token {
                break;
            }
            input = vec![token];
        }
        Ok((output, prefill_seconds, decode_seconds, cache))
    }
}
impl Generator for QwenGenerator {
    fn generate(&mut self, messages: &[Value], max_new_tokens: usize) -> Result<Generation> {
        let start = Instant::now();
        let ids = self.tokenizer.encode_chat(messages, true)?;
        let prefix = if self.options.prefix_cache
            && messages.first().is_some_and(|m| m["role"] == "system")
        {
            Some(self.tokenizer.encode_chat(&messages[..1], false)?)
        } else {
            None
        };
        let tokenize_seconds = start.elapsed().as_secs_f64();
        let (tokens, prefill, decode, cache) =
            self.generate_tokens_with_prefix(&ids, max_new_tokens, prefix.as_deref())?;
        let terminated = tokens.last() == Some(&self.tokenizer.eos_token);
        Ok(Generation {
            text: self.tokenizer.decode(&tokens)?,
            input_tokens: ids.len(),
            output_tokens: tokens.len(),
            metadata: json!({"backend":"candle-0.9.1-qwen2","precision":self.options.precision,
                "device":self.options.device,"lora_merged":true,
                "generation_config_sha256":BASE_GENERATION_CONFIG_SHA256,
                "repetition_penalty":self.repetition_penalty,
                "effective_generation_config":{"do_sample":false,"bos_token_id":151643,
                    "pad_token_id":151643,"eos_token_id":151645,"repetition_penalty":self.repetition_penalty,
                    "max_new_tokens":max_new_tokens},
                "prefix_cache_enabled":self.options.prefix_cache,"prefix_cache_hit":cache.hit,
                "prefix_cache_tokens":cache.tokens,"prefix_forward_seconds":cache.prefix_forward_seconds,
                "base_model":self.identity["base_model"],"base_safetensors_sha256":BASE_WEIGHTS_SHA256,
                "bundle_verification":self.bundle_verification,
                "adapter_model_sha256":self.adapter_hash,"tiny":false,"eos_terminated":terminated,
                "generation_limit_reached":tokens.len() >= max_new_tokens && !terminated,
                "tokenize_seconds":tokenize_seconds,"prefill_seconds":prefill,"decode_seconds":decode,
                "decode_forward_seconds":decode,"decode_tokens":tokens.len().saturating_sub(1),
                "generated_token_ids":tokens}),
        })
    }
}

pub fn select_device(name: &str) -> Result<Device> {
    match name {
        "cpu" => Ok(Device::Cpu),
        "metal" => {
            #[cfg(feature = "metal")]
            {
                Ok(Device::new_metal(0)?)
            }
            #[cfg(not(feature = "metal"))]
            {
                bail!("Metal requires building with --features metal")
            }
        }
        other => bail!("unsupported device {other}, choose cpu or metal"),
    }
}
fn validate_adapter_names(
    adapters: &HashMap<String, Tensor>,
    config: &Config,
    rank: usize,
) -> Result<()> {
    let layers = config.num_hidden_layers;
    ensure!(
        adapters.len() == layers * 4,
        "unexpected adapter tensor count"
    );
    for layer in 0..layers {
        for module in ["q_proj", "v_proj"] {
            let stem = format!("base_model.model.model.layers.{layer}.self_attn.{module}");
            let a = adapters
                .get(&format!("{stem}.lora_A.weight"))
                .context("missing LoRA A")?;
            let b = adapters
                .get(&format!("{stem}.lora_B.weight"))
                .context("missing LoRA B")?;
            let output = if module == "q_proj" {
                config.hidden_size
            } else {
                config.num_key_value_heads * (config.hidden_size / config.num_attention_heads)
            };
            ensure!(
                a.dims2()? == (rank, config.hidden_size) && b.dims2()? == (output, rank),
                "adapter shape differs from configured rank/projection at {stem}"
            );
            for matrix in ["A", "B"] {
                ensure!(adapters.contains_key(&format!("base_model.model.model.layers.{layer}.self_attn.{module}.lora_{matrix}.weight")),
                    "missing adapter tensor for layer {layer} {module} {matrix}");
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sha2::{Digest, Sha256};
    struct SmallWeights;
    impl SimpleBackend for SmallWeights {
        fn get(
            &self,
            shape: Shape,
            name: &str,
            _: Init,
            dtype: DType,
            dev: &Device,
        ) -> candle_core::Result<Tensor> {
            let n = shape.elem_count();
            let values = if name.contains("layernorm") || name == "model.norm.weight" {
                vec![1f32; n]
            } else {
                let salt = name.bytes().map(u32::from).sum::<u32>() as usize;
                (0..n)
                    .map(|i| (((i + salt) % 29) as f32 - 14.) * 0.017)
                    .collect()
            };
            Tensor::from_vec(values, shape, dev)?.to_dtype(dtype)
        }
        fn contains_tensor(&self, name: &str) -> bool {
            name != "lm_head.weight"
        }
    }
    fn small_config() -> Config {
        Config {
            vocab_size: 17,
            hidden_size: 8,
            intermediate_size: 16,
            num_hidden_layers: 2,
            num_attention_heads: 2,
            num_key_value_heads: 1,
            max_position_embeddings: 32,
            sliding_window: 32,
            max_window_layers: 2,
            tie_word_embeddings: true,
            rope_theta: 10000.,
            rms_norm_eps: 1e-6,
            use_sliding_window: false,
            hidden_act: candle_nn::Activation::Silu,
        }
    }
    fn tiny_generator(prefix_cache: bool) -> QwenGenerator {
        let cfg = small_config();
        let vb = VarBuilder::from_backend(Box::new(SmallWeights), DType::F32, Device::Cpu);
        let model = ModelForCausalLM::new(&cfg, vb).unwrap();
        QwenGenerator {
            model,
            tokenizer: ChatTokenizer {
                tokenizer: Tokenizer::new(tokenizers::models::bpe::BPE::default()),
                template: String::new(),
                eos_token: u32::MAX,
            },
            device: Device::Cpu,
            options: LoadOptions {
                prefix_cache,
                ..Default::default()
            },
            identity: json!({}),
            bundle_verification: json!({"test_fixture":true}),
            adapter_hash: String::new(),
            load_seconds: 0.,
            context_limit: 32,
            prefix_snapshot: None,
            repetition_penalty: 1.1,
        }
    }
    #[test]
    fn merged_projection_matches_independent_matrix_arithmetic() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("base.safetensors");
        let base = HashMap::from([(
            "model.layers.0.self_attn.q_proj.weight".to_string(),
            Tensor::new(&[[1f32, 2.], [3., 4.]], &Device::Cpu).unwrap(),
        )]);
        candle_core::safetensors::save(&base, &path).unwrap();
        let stem = "base_model.model.model.layers.0.self_attn.q_proj";
        let adapters = HashMap::from([
            (
                format!("{stem}.lora_A.weight"),
                Tensor::new(&[[1f32, 2.]], &Device::Cpu).unwrap(),
            ),
            (
                format!("{stem}.lora_B.weight"),
                Tensor::new(&[[0.5f32], [1.]], &Device::Cpu).unwrap(),
            ),
        ]);
        let backend = MergedBackend {
            base: candle_core::safetensors::BufferedSafetensors::new(fs::read(path).unwrap())
                .unwrap(),
            adapters,
            scale: 2.,
        };
        let merged = backend
            .get(
                (2, 2).into(),
                "model.layers.0.self_attn.q_proj.weight",
                Init::Const(0.),
                DType::F32,
                &Device::Cpu,
            )
            .unwrap();
        assert_eq!(
            merged.to_vec2::<f32>().unwrap(),
            vec![vec![2., 4.], vec![5., 8.]]
        );
        let input = Tensor::new(&[[2f32, 3.]], &Device::Cpu).unwrap();
        let actual = input.matmul(&merged.t().unwrap()).unwrap();
        assert_eq!(actual.to_vec2::<f32>().unwrap(), vec![vec![16., 34.]]);
    }
    #[test]
    fn lora_configuration_rejects_unsupported_and_duplicate_modules() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir(dir.path().join("adapter")).unwrap();
        let mut cfg = json!({"peft_type":"LORA","task_type":"CAUSAL_LM", "base_model_name_or_path":BASE_MODEL,
            "revision":BASE_REVISION,"rank_pattern":{},"alpha_pattern":{},"r":8,"lora_alpha":16,
            "lora_dropout":0.0,"target_modules":["q_proj","v_proj"],"fan_in_fan_out":false,"bias":"none"});
        let path = dir.path().join("adapter/adapter_config.json");
        fs::write(&path, serde_json::to_vec(&cfg).unwrap()).unwrap();
        assert!(LoraConfig::load(dir.path()).is_ok());
        for modules in [json!(["q_proj", "o_proj"]), json!(["q_proj", "q_proj"])] {
            cfg["target_modules"] = modules;
            fs::write(&path, serde_json::to_vec(&cfg).unwrap()).unwrap();
            assert!(LoraConfig::load(dir.path()).is_err());
        }
    }
    #[test]
    fn adapter_dimensions_must_match_configured_rank_and_projection() {
        let cfg = small_config();
        let mut adapters = HashMap::new();
        for layer in 0..cfg.num_hidden_layers {
            for module in ["q_proj", "v_proj"] {
                let stem = format!("base_model.model.model.layers.{layer}.self_attn.{module}");
                let output = if module == "q_proj" {
                    cfg.hidden_size
                } else {
                    cfg.num_key_value_heads * (cfg.hidden_size / cfg.num_attention_heads)
                };
                adapters.insert(
                    format!("{stem}.lora_A.weight"),
                    Tensor::zeros((1, cfg.hidden_size), DType::F32, &Device::Cpu).unwrap(),
                );
                adapters.insert(
                    format!("{stem}.lora_B.weight"),
                    Tensor::zeros((output, 1), DType::F32, &Device::Cpu).unwrap(),
                );
            }
        }
        assert!(validate_adapter_names(&adapters, &cfg, 1).is_ok());
        adapters.insert(
            "base_model.model.model.layers.0.self_attn.q_proj.lora_A.weight".into(),
            Tensor::zeros((1, cfg.hidden_size - 1), DType::F32, &Device::Cpu).unwrap(),
        );
        assert!(validate_adapter_names(&adapters, &cfg, 1).is_err());
    }
    #[test]
    fn production_generation_budget_is_checked_before_tensor_creation() {
        let mut generator = tiny_generator(false);
        assert!(generator.generate_tokens(&[1, 2], 0).is_err());
        assert!(generator.generate_tokens(&[1, 2], 513).is_err());
        assert!(generator.generate_tokens(&[1; 31], 2).is_err());
        generator.context_limit = 32768;
        assert!(generator.generate_tokens(&[1; 4096], 1).is_err());
    }
    #[test]
    fn pinned_greedy_configuration_keeps_repetition_penalty() {
        let bytes = include_bytes!("fixtures/qwen-generation-config.json");
        assert_eq!(
            format!("{:x}", Sha256::digest(bytes)),
            BASE_GENERATION_CONFIG_SHA256
        );
        let config: Value = serde_json::from_slice(bytes).unwrap();
        assert_eq!(config["repetition_penalty"], json!(1.1));
        assert_eq!(
            config["repetition_penalty"].as_f64().unwrap() as f32,
            1.1f32
        );
    }
    #[test]
    fn repetition_penalty_uses_prompt_and_generated_tokens_once_with_sign() {
        let logits = Tensor::new(&[4f32, -3., 0., 8., -5., 6.], &Device::Cpu).unwrap();
        let prompt = [0u32, 1, 1, 2];
        let generated = [3u32, 3];
        let context = [prompt.as_slice(), generated.as_slice()].concat();
        let processed =
            candle_transformers::utils::apply_repeat_penalty(&logits, 2., &context).unwrap();
        assert_eq!(
            processed.to_vec1::<f32>().unwrap(),
            vec![2., -6., 0., 4., -5., 6.]
        );
        // A generated token changes the next decision even when absent from the prompt.
        let logits = Tensor::new(&[8f32, 7., 6., 5.], &Device::Cpu).unwrap();
        let processed =
            candle_transformers::utils::apply_repeat_penalty(&logits, 2., &[2, 0]).unwrap();
        assert_eq!(processed.argmax(0).unwrap().to_scalar::<u32>().unwrap(), 1);
        let tied = Tensor::new(&[2f32, 2., 1.], &Device::Cpu).unwrap();
        assert_eq!(tied.argmax(0).unwrap().to_scalar::<u32>().unwrap(), 0);
    }
    #[test]
    fn cached_system_state_preserves_new_user_and_changed_system_generations() {
        let mut cached = tiny_generator(true);
        let mut uncached = tiny_generator(false);
        for (ids, prefix, expect_hit) in [
            (vec![1, 2, 3, 4, 5], vec![1, 2], false),
            (vec![1, 2, 7, 8, 9], vec![1, 2], true),
            (vec![10, 11, 7, 8, 9], vec![10, 11], false),
        ] {
            let expected = uncached.generate_tokens(&ids, 4).unwrap().0;
            let (actual, _, _, stats) = cached
                .generate_tokens_with_prefix(&ids, 4, Some(&prefix))
                .unwrap();
            assert_eq!(actual, expected);
            assert_eq!(stats.hit, expect_hit);
            assert_eq!(stats.tokens, prefix.len());
        }
        // No trusted message boundary is supplied by the raw-token interface.
        cached.generate_tokens(&[1, 2, 3], 2).unwrap();
        assert!(cached.prefix_snapshot.is_none());
    }
    #[test]
    fn failed_forward_clears_request_cache_and_preserves_system_snapshot() {
        let mut generator = tiny_generator(true);
        let prefix = [1u32, 2];
        generator
            .generate_tokens_with_prefix(&[1, 2, 3, 4], 2, Some(&prefix))
            .unwrap();
        // Token 17 lies outside this independently authored 17-token model.
        assert!(
            generator
                .generate_tokens_with_prefix(&[1, 2, 17], 2, Some(&prefix))
                .is_err()
        );
        assert_eq!(generator.prefix_snapshot.as_ref().unwrap().tokens, prefix);
        // A direct offset-zero causal pass succeeds only when the active cache is empty.
        let probe = Tensor::new(&[[3u32, 4]], &Device::Cpu).unwrap();
        assert!(generator.model.forward(&probe, 0).is_ok());
        generator.model.clear_kv_cache();
        let ids = [1u32, 2, 7, 8];
        let actual = generator
            .generate_tokens_with_prefix(&ids, 4, Some(&prefix))
            .unwrap();
        let expected = tiny_generator(false).generate_tokens(&ids, 4).unwrap();
        assert_eq!(actual.0, expected.0);
        assert!(actual.3.hit);
    }
    #[test]
    fn system_cache_requires_exact_bounded_prefix_and_remaining_prompt() {
        assert!(eligible_system_prefix(&[1, 2, 3, 4], &[1, 2]));
        // A different user suffix can reuse the same system prefix.
        assert!(eligible_system_prefix(&[1, 2, 9, 8], &[1, 2]));
        assert!(!eligible_system_prefix(&[1, 7, 3], &[1, 2]));
        assert!(!eligible_system_prefix(&[1, 2], &[1, 2]));
        assert!(!eligible_system_prefix(&[1, 2], &[]));
        assert!(!eligible_system_prefix(&[1; 1026], &[1; 1025]));
    }
    #[test]
    fn original_jinja_preserves_system_and_message_boundaries() {
        let tokenizer = ChatTokenizer {
            tokenizer: Tokenizer::new(tokenizers::models::bpe::BPE::default()),
            template: include_str!("fixtures/qwen-chat.jinja").into(),
            eos_token: 151645,
        };
        let rendered = tokenizer
            .render(
                &[
                    json!({"role":"system","content":"Rules."}),
                    json!({"role":"user","content":"Use α and \"quoted\" text.\nNext."}),
                ],
                true,
            )
            .unwrap();
        assert_eq!(
            rendered,
            "<|im_start|>system\nRules.<|im_end|>\n<|im_start|>user\nUse α and \"quoted\" text.\nNext.<|im_end|>\n<|im_start|>assistant\n"
        );
    }
    #[test]
    fn original_jinja_groups_tool_responses() {
        let tokenizer = ChatTokenizer {
            tokenizer: Tokenizer::new(tokenizers::models::bpe::BPE::default()),
            template: include_str!("fixtures/qwen-chat.jinja").into(),
            eos_token: 151645,
        };
        let rendered = tokenizer
            .render(
                &[
                    json!({"role":"user","content":"Inspect"}),
                    json!({"role":"tool","content":"one"}),
                    json!({"role":"tool","content":"two"}),
                ],
                true,
            )
            .unwrap();
        assert!(rendered.ends_with("<|im_start|>user\n<tool_response>\none\n</tool_response>\n<tool_response>\ntwo\n</tool_response><|im_end|>\n<|im_start|>assistant\n"));
    }
    #[test]
    fn rank_scale_matches_peft() {
        let cfg = LoraConfig {
            r: 8,
            lora_alpha: 16.,
            lora_dropout: 0.,
            target_modules: vec!["q_proj".into(), "v_proj".into()],
            fan_in_fan_out: false,
            bias: "none".into(),
            use_dora: false,
            use_rslora: false,
        };
        assert_eq!(cfg.scale(), 2.);
    }
    #[test]
    fn pinned_model_identity_is_explicit() {
        assert_eq!(BASE_REVISION.len(), 40);
        assert_eq!(BASE_WEIGHTS_SHA256.len(), 64);
    }
}
