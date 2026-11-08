//! Explicit CLI entry points. Nothing is loaded into an external MCP client.
#![forbid(unsafe_code)]
use anyhow::{Context, Result, anyhow, ensure};
use clap::{Args, Parser, Subcommand};
use eal_model_host::{
    bundle,
    contract::{Generator, RunOptions, SessionConfig},
    host::{NativeSession, run_policy},
    model::{LoadOptions, QwenGenerator},
};
use serde_json::{Value, json};
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

#[derive(Parser)]
#[command(
    version,
    about = "Run the pinned local tool-use model through an operator-configured EAL MCP session"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Verify all pinned model and adapter bytes without loading tensors or starting MCP.
    Verify {
        #[arg(long)]
        base: PathBuf,
        #[arg(long)]
        adapter: PathBuf,
    },
    /// Run one task. The output trace preserves native responses and stopping status.
    Run(RunArgs),
    /// Generate one response offline, without starting an MCP process.
    Generate(GenerateArgs),
}

#[derive(Args)]
struct ModelArgs {
    #[arg(long)]
    base: PathBuf,
    #[arg(long)]
    adapter: PathBuf,
    #[arg(long, default_value = "cpu", value_parser = ["cpu", "metal"])]
    device: String,
    #[arg(long, default_value = "f32", value_parser = ["f32"])]
    precision: String,
    /// Disable reuse of the exact shared system prefix between model decisions.
    #[arg(long)]
    no_prefix_cache: bool,
}
impl ModelArgs {
    fn load(&self) -> Result<QwenGenerator> {
        QwenGenerator::load(
            &self.base,
            &self.adapter,
            LoadOptions {
                precision: self.precision.clone(),
                device: self.device.clone(),
                prefix_cache: !self.no_prefix_cache,
            },
        )
    }
    fn configuration(&self) -> Value {
        json!({"device":self.device,"precision":self.precision,"prefix_cache":!self.no_prefix_cache})
    }
}

#[derive(Args)]
struct RunArgs {
    #[arg(long)]
    task: PathBuf,
    #[command(flatten)]
    model: ModelArgs,
    #[arg(long)]
    native_executable: PathBuf,
    #[arg(long)]
    workspace: PathBuf,
    #[arg(long)]
    output: PathBuf,
    #[arg(long)]
    registry: Option<PathBuf>,
    /// Operator-selected registered entries. Repeating this flag selects registered exposure.
    #[arg(long)]
    known_entry: Vec<String>,
    #[arg(long, default_value_t = 30.0)]
    timeout_secs: f64,
    #[arg(long, default_value_t = 6)]
    max_steps: usize,
    #[arg(long, default_value_t = 160)]
    max_new_tokens: usize,
    /// Opt into observational catalogue scope with retained evidence identifiers.
    #[arg(long)]
    evidence_guided: bool,
}

#[derive(Args)]
struct GenerateArgs {
    /// JSON array of messages with string role and content fields.
    #[arg(long)]
    messages: PathBuf,
    #[command(flatten)]
    model: ModelArgs,
    #[arg(long)]
    output: PathBuf,
    #[arg(long, default_value_t = 160)]
    max_new_tokens: usize,
}

fn read_json(path: &Path) -> Result<Value> {
    const MAX_BYTES: usize = 4 * 1024 * 1024;
    let mut bytes = Vec::new();
    File::open(path)
        .with_context(|| format!("open {}", path.display()))?
        .take((MAX_BYTES + 1) as u64)
        .read_to_end(&mut bytes)?;
    eal_collector_contracts::strict_json_exact(
        &bytes,
        eal_collector_contracts::JsonLimits {
            max_bytes: MAX_BYTES,
            max_depth: 128,
            max_nodes: 100_000,
        },
    )
    .map_err(|error| anyhow!("read {}: {error}", path.display()))
}

/// Reserve a fresh output before inference or acquisition can have effects.
fn reserve_output(path: &Path) -> Result<File> {
    if let Some(parent) = path.parent().filter(|path| !path.as_os_str().is_empty()) {
        fs::create_dir_all(parent)?;
    }
    OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .with_context(|| format!("output must be a new file: {}", path.display()))
}

fn retain_output(mut output: File, report: &Value) -> Result<()> {
    serde_json::to_writer_pretty(&mut output, report)?;
    output.write_all(b"\n")?;
    output.flush()?;
    Ok(())
}

fn validate_generation_limit(value: usize) -> Result<()> {
    ensure!(
        (1..=512).contains(&value),
        "max-new-tokens must be between 1 and 512"
    );
    Ok(())
}

async fn run(args: RunArgs) -> Result<()> {
    ensure!(
        (1..=32).contains(&args.max_steps),
        "max-steps must be between 1 and 32"
    );
    validate_generation_limit(args.max_new_tokens)?;
    ensure!(
        args.timeout_secs.is_finite() && args.timeout_secs > 0.0 && args.timeout_secs <= 3600.0,
        "timeout-secs must be positive, finite and at most 3600"
    );
    let task = read_json(&args.task)?;
    ensure!(
        task["request"].is_string() && task["state"].is_object(),
        "task requires a request string and a state object"
    );
    let output = reserve_output(&args.output)?;
    let started = Instant::now();
    let mut report = json!({"schema":"eal.local-model-run/1","configuration":args.model.configuration(),
        "max_steps":args.max_steps,"max_new_tokens":args.max_new_tokens,"evidence_guided":args.evidence_guided});
    let result: Result<()> = async {
        let mut generator = args.model.load()?;
        report["bundle"] = generator.bundle_identity().clone();
        report["model_load_seconds"] = json!(generator.load_seconds);
        report["configuration"]["repetition_penalty"] =
            json!(generator.effective_repetition_penalty());
        fs::create_dir_all(&args.workspace)
            .with_context(|| format!("create operator workspace {}", args.workspace.display()))?;
        let mut config = SessionConfig::new(args.native_executable, args.workspace);
        config.registry = args.registry;
        config.known_entries = args.known_entry;
        config.timeout = Duration::try_from_secs_f64(args.timeout_secs)?;
        let mut session = NativeSession::connect(config).await?;
        report["initialization"] = session.initialization.clone();
        report["catalogue"] = json!(session.tools);
        let options = RunOptions {
            max_steps: args.max_steps,
            max_new_tokens: args.max_new_tokens,
            evidence_guided: args.evidence_guided,
            ..RunOptions::default()
        };
        // Close after every returned Result, including admission or generator failures.
        let run_result = run_policy(&mut session, &task, &mut generator, &options).await;
        let close_result = session.close().await;
        match run_result {
            Ok(run) => report["run"] = run,
            Err(error) => {
                if let Err(close_error) = close_result {
                    report["close_error"] = json!(close_error.to_string());
                }
                return Err(error);
            }
        }
        if let Err(error) = close_result {
            report["close_error"] = json!(error.to_string());
            return Err(error.context("MCP session closure failed after the retained execution"));
        }
        Ok(())
    }
    .await;
    report["elapsed_seconds"] = json!(started.elapsed().as_secs_f64());
    report["status"] = json!(if result.is_ok() { "recorded" } else { "failed" });
    if let Err(error) = &result {
        report["error"] = json!(format!("{error:#}"));
    }
    let stop = report["run"]["stop"].clone();
    retain_output(output, &report)?;
    println!(
        "{}",
        json!({"output":args.output,"status":report["status"],"stop":stop})
    );
    result
}

fn generate(args: GenerateArgs) -> Result<()> {
    validate_generation_limit(args.max_new_tokens)?;
    let messages = read_json(&args.messages)?;
    let messages = messages
        .as_array()
        .context("messages file must contain a JSON array")?;
    ensure!(
        !messages.is_empty()
            && messages
                .iter()
                .all(|message| message["role"].is_string() && message["content"].is_string()),
        "messages must be a nonempty array with string role and content fields"
    );
    let output = reserve_output(&args.output)?;
    let started = Instant::now();
    let mut report = json!({"schema":"eal.local-generation/1","configuration":args.model.configuration(),"max_new_tokens":args.max_new_tokens});
    let result: Result<()> = (|| {
        let mut generator = args.model.load()?;
        report["bundle"] = generator.bundle_identity().clone();
        report["model_load_seconds"] = json!(generator.load_seconds);
        report["configuration"]["repetition_penalty"] =
            json!(generator.effective_repetition_penalty());
        report["generation"] =
            serde_json::to_value(generator.generate(messages, args.max_new_tokens)?)?;
        Ok(())
    })();
    report["elapsed_seconds"] = json!(started.elapsed().as_secs_f64());
    report["status"] = json!(if result.is_ok() { "recorded" } else { "failed" });
    if let Err(error) = &result {
        report["error"] = json!(format!("{error:#}"));
    }
    retain_output(output, &report)?;
    println!(
        "{}",
        json!({"output":args.output,"status":report["status"]})
    );
    result
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    match Cli::parse().command {
        Command::Verify { base, adapter } => {
            let report = bundle::verify_pinned_bundle(&base, &adapter)?;
            println!("{}", serde_json::to_string_pretty(&report)?);
            Ok(())
        }
        Command::Run(args) => run(args).await,
        Command::Generate(args) => generate(args),
    }
}
