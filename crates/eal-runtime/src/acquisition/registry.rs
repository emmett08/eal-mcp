use super::{
    AcquisitionError, AcquisitionResult, MAX_OUTPUT_BYTES, PinnedFile, ToolBinding, ToolKind,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};
use toml::Value;

#[derive(Clone, Debug, Default)]
pub struct ToolRegistry {
    pub bindings: BTreeMap<String, ToolBinding>,
}
impl ToolRegistry {
    pub fn new(bindings: BTreeMap<String, ToolBinding>) -> Result<Self, AcquisitionError> {
        for (name, binding) in &bindings {
            if name != &binding.name {
                return Err("Registry keys must match their binding names".into());
            }
            validate_binding(binding)?;
        }
        Ok(Self { bindings })
    }
    pub fn load(path: impl AsRef<Path>) -> Result<Self, AcquisitionError> {
        let path = super::files::resolve(path.as_ref())?;
        let contents =
            super::files::read_regular(&path, 1024 * 1024, "The registry must be a regular file")?;
        let text = std::str::from_utf8(&contents)
            .map_err(|_| AcquisitionError::invalid("The registry must contain UTF-8 TOML"))?;
        Self::from_toml(text)
    }
    pub fn from_toml(text: &str) -> Result<Self, AcquisitionError> {
        if text.len() > 1024 * 1024 {
            return Err("The tool registry exceeds its 1 MiB input limit".into());
        }
        let document: toml::Table =
            toml::from_str(text).map_err(|e| AcquisitionError::invalid(e.to_string()))?;
        if document.keys().any(|k| k != "tools")
            || document.get("tools").is_some_and(|v| !v.is_table())
        {
            return Err("The registry must contain only a [tools] table".into());
        }
        let mut bindings = BTreeMap::new();
        if let Some(Value::Table(tools)) = document.get("tools") {
            for (name, raw) in tools {
                bindings.insert(name.clone(), parse_binding(name, raw)?);
            }
        }
        Self::new(bindings)
    }
    pub fn binding_for(&self, name: &str, version: &str) -> Result<&ToolBinding, AcquisitionError> {
        let binding = self.bindings.get(name).ok_or_else(|| {
            let repr = eal_core::expressions::diagnostic_repr(
                &serde_json::json!(name),
                &eal_core::ExecutionLimits::default(),
            )
            .unwrap_or_else(|_| format!("{name:?}"));
            AcquisitionError::invalid(format!("Tool {repr} is not in the operator registry"))
        })?;
        if binding.version != version {
            return Err("Declared tool version differs from the operator registry".into());
        }
        Ok(binding)
    }
    pub fn acquire(
        &self,
        binding: &ToolBinding,
        request: &serde_json::Value,
        workspace: &Path,
        secret: &[u8],
        supervisor: &crate::process::Supervisor,
        lease: Option<&crate::lease::AcquisitionLease>,
    ) -> Result<AcquisitionResult, AcquisitionError> {
        super::acquire(binding, request, workspace, secret, supervisor, lease)
    }
    // Keep the explicit policy, lease and supervisor inputs alongside the source-compatible adapter call.
    #[allow(clippy::too_many_arguments)]
    pub fn acquire_with_limits(
        &self,
        binding: &ToolBinding,
        request: &serde_json::Value,
        workspace: &Path,
        secret: &[u8],
        supervisor: &crate::process::Supervisor,
        lease: Option<&crate::lease::AcquisitionLease>,
        limits: &eal_core::ExecutionLimits,
    ) -> Result<AcquisitionResult, AcquisitionError> {
        super::acquire_with_limits(
            binding, request, workspace, secret, supervisor, lease, limits,
        )
    }
}

fn parse_binding(name: &str, raw: &Value) -> Result<ToolBinding, AcquisitionError> {
    let allowed = [
        "kind",
        "version",
        "argv",
        "path",
        "timeout_seconds",
        "max_output_bytes",
        "env",
        "pinned_files",
        "parallel_safe",
        "inherit_env",
    ];
    let raw = raw
        .as_table()
        .filter(|v| v.keys().all(|k| allowed.contains(&k.as_str())))
        .ok_or_else(|| {
            AcquisitionError::invalid(format!("Unknown registry settings for {name}"))
        })?;
    let error = |message: &str| AcquisitionError::invalid(format!("{name}: {message}"));
    let kind = match raw.get("kind").and_then(Value::as_str) {
        Some("command") => ToolKind::Command,
        Some("json_file") => ToolKind::JsonFile,
        _ => return Err(error("kind must be command or json_file")),
    };
    let version = raw
        .get("version")
        .and_then(Value::as_str)
        .filter(|v| !v.is_empty())
        .ok_or_else(|| error("a nonempty version is required"))?
        .to_owned();
    let argv = match raw.get("argv") {
        None => Vec::new(),
        Some(Value::Array(values)) => values
            .iter()
            .map(|v| {
                v.as_str()
                    .filter(|s| !s.is_empty())
                    .map(str::to_owned)
                    .ok_or_else(|| error("argv must contain nonempty strings"))
            })
            .collect::<Result<Vec<_>, _>>()?,
        Some(_) => return Err(error("argv must contain nonempty strings")),
    };
    let path = raw.get("path").and_then(Value::as_str).map(str::to_owned);
    if kind == ToolKind::Command && (argv.is_empty() || raw.contains_key("path")) {
        return Err(error("command requires argv and does not use path"));
    }
    if kind == ToolKind::JsonFile && (path.as_deref().is_none_or(str::is_empty) || !argv.is_empty())
    {
        return Err(error("json_file requires a path and does not use argv"));
    }
    let timeout_seconds = match raw.get("timeout_seconds") {
        None => 30.0,
        Some(Value::Integer(v)) => *v as f64,
        Some(Value::Float(v)) => *v,
        Some(_) => {
            return Err(error(
                "timeout_seconds must be finite, positive and at most 3600",
            ));
        }
    };
    if !timeout_seconds.is_finite()
        || !(0.0..=3600.0).contains(&timeout_seconds)
        || timeout_seconds == 0.0
    {
        return Err(error(
            "timeout_seconds must be finite, positive and at most 3600",
        ));
    }
    let max_output_bytes = match raw.get("max_output_bytes") {
        None => 1024 * 1024,
        Some(Value::Integer(v)) if (1..=MAX_OUTPUT_BYTES as i64).contains(v) => *v as usize,
        Some(_) => {
            return Err(error(
                "max_output_bytes must be an integer from 1 through 16777216",
            ));
        }
    };
    let env = match raw.get("env") {
        None => BTreeMap::new(),
        Some(Value::Table(values)) => values
            .iter()
            .map(|(k, v)| {
                v.as_str()
                    .map(|v| (k.clone(), v.into()))
                    .ok_or_else(|| error("env must map strings to strings"))
            })
            .collect::<Result<BTreeMap<_, _>, _>>()?,
        Some(_) => return Err(error("env must map strings to strings")),
    };
    let parallel_safe = match raw.get("parallel_safe") {
        None => false,
        Some(Value::Boolean(v)) => *v,
        Some(_) => return Err(error("parallel_safe must be a boolean")),
    };
    let inherit_error =
        || error("inherit_env requires up to 64 distinct command environment variable names");
    let inherit_env = match raw.get("inherit_env") {
        None => None,
        Some(Value::Array(values)) if kind == ToolKind::Command && values.len() <= 64 => {
            let values = values
                .iter()
                .map(|v| {
                    v.as_str()
                        .filter(|v| environment_name(v))
                        .map(str::to_owned)
                        .ok_or_else(inherit_error)
                })
                .collect::<Result<Vec<_>, _>>()?;
            if values.iter().collect::<BTreeSet<_>>().len() != values.len() {
                return Err(inherit_error());
            }
            Some(values)
        }
        Some(_) => return Err(inherit_error()),
    };
    let pin_error = || error("pinned_files requires distinct paths and lowercase SHA-256 digests");
    let pinned_files = match raw.get("pinned_files") {
        None => Vec::new(),
        Some(Value::Array(values)) if values.len() <= 32 => {
            let values = values
                .iter()
                .map(|value| {
                    let value = value
                        .as_table()
                        .filter(|v| {
                            v.len() == 2 && v.contains_key("path") && v.contains_key("sha256")
                        })
                        .ok_or_else(pin_error)?;
                    let path = value["path"]
                        .as_str()
                        .filter(|v| !v.is_empty())
                        .ok_or_else(pin_error)?;
                    let sha256 = value["sha256"]
                        .as_str()
                        .filter(|v| valid_digest(v))
                        .ok_or_else(pin_error)?;
                    Ok(PinnedFile {
                        path: path.into(),
                        sha256: sha256.into(),
                    })
                })
                .collect::<Result<Vec<_>, AcquisitionError>>()?;
            if values
                .iter()
                .map(|v| &v.path)
                .collect::<BTreeSet<_>>()
                .len()
                != values.len()
            {
                return Err(pin_error());
            }
            values
        }
        Some(_) => return Err(pin_error()),
    };
    if kind != ToolKind::Command && !pinned_files.is_empty() {
        return Err(error("only command bindings can pin executable files"));
    }
    Ok(ToolBinding {
        name: name.into(),
        kind,
        version,
        argv,
        path,
        timeout_seconds,
        max_output_bytes,
        env,
        pinned_files,
        parallel_safe,
        inherit_env,
    })
}

pub(super) fn validate_binding(binding: &ToolBinding) -> Result<(), AcquisitionError> {
    let mut raw = toml::Table::new();
    raw.insert("kind".into(), Value::String(binding.kind.as_str().into()));
    raw.insert("version".into(), Value::String(binding.version.clone()));
    raw.insert(
        "argv".into(),
        Value::Array(binding.argv.iter().cloned().map(Value::String).collect()),
    );
    if let Some(path) = &binding.path {
        raw.insert("path".into(), Value::String(path.clone()));
    }
    raw.insert(
        "timeout_seconds".into(),
        Value::Float(binding.timeout_seconds),
    );
    raw.insert(
        "max_output_bytes".into(),
        Value::Integer(i64::try_from(binding.max_output_bytes).unwrap_or(i64::MAX)),
    );
    raw.insert(
        "env".into(),
        Value::Table(
            binding
                .env
                .iter()
                .map(|(k, v)| (k.clone(), Value::String(v.clone())))
                .collect(),
        ),
    );
    raw.insert(
        "parallel_safe".into(),
        Value::Boolean(binding.parallel_safe),
    );
    if let Some(values) = &binding.inherit_env {
        raw.insert(
            "inherit_env".into(),
            Value::Array(values.iter().cloned().map(Value::String).collect()),
        );
    }
    raw.insert(
        "pinned_files".into(),
        Value::Array(
            binding
                .pinned_files
                .iter()
                .map(|p| {
                    Value::Table(
                        [
                            ("path".into(), Value::String(p.path.clone())),
                            ("sha256".into(), Value::String(p.sha256.clone())),
                        ]
                        .into_iter()
                        .collect(),
                    )
                })
                .collect(),
        ),
    );
    parse_binding(&binding.name, &Value::Table(raw)).map(|_| ())
}
fn valid_digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|v| v.is_ascii_digit() || (b'a'..=b'f').contains(&v))
}
fn environment_name(value: &str) -> bool {
    let mut bytes = value.bytes();
    bytes
        .next()
        .is_some_and(|b| b.is_ascii_alphabetic() || b == b'_')
        && bytes.all(|b| b.is_ascii_alphanumeric() || b == b'_')
}
