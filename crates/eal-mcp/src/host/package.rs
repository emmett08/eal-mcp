//! Translate a portable source package into the ordinary MCP execute request.
use serde_json::json;
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};

#[derive(Debug)]
pub(super) struct PackageInvocation {
    pub args: Vec<String>,
    pub request: Option<String>,
    pub output_tools: Option<(PathBuf, PathBuf)>,
}

/// No alternate interpreter: this only reads bounded files and builds the same
/// closed operation request accepted from a text model or native MCP client.
pub(super) fn prepare(argv: Vec<String>) -> Result<PackageInvocation, String> {
    let mode = argv.first().map(String::as_str).unwrap_or("");
    if !matches!(mode, "run" | "prepare") {
        return Ok(PackageInvocation {
            args: argv,
            request: None,
            output_tools: None,
        });
    }
    let source_path = argv
        .get(1)
        .filter(|path| !path.starts_with('-'))
        .ok_or("run/prepare requires an EAL source file")?;
    let source = read_text(Path::new(source_path), super::MAX_REQUEST_BYTES)?;
    let mut settings = Vec::new();
    let mut request = json!({"operation":if mode == "prepare" {"prepare"} else {"execute"},"source":source,"context":{}});
    let mut output_tools = None;
    let mut seen = BTreeSet::new();
    let mut index = 2;
    while index < argv.len() {
        let (flag, inline) = argv[index]
            .split_once('=')
            .map_or((argv[index].as_str(), None), |(flag, value)| {
                (flag, Some(value))
            });
        if !seen.insert(flag.to_owned()) {
            return Err(format!("Repeated package option {flag}"));
        }
        if !matches!(
            flag,
            "--tools"
                | "--output-tools"
                | "--workspace"
                | "--context"
                | "--claim"
                | "--now"
                | "--timeout"
                | "--methods"
                | "--limits"
                | "--database"
        ) {
            return Err(format!("Unknown package option {flag}"));
        }
        let value = match inline {
            Some(value) => value,
            None => {
                index += 1;
                argv.get(index)
                    .ok_or_else(|| format!("Missing value for {flag}"))?
            }
        };
        if value.is_empty() {
            return Err(format!("Empty value for {flag}"));
        }
        match flag {
            "--output-tools" => {
                if mode != "prepare" {
                    return Err("--output-tools belongs to prepare".into());
                }
                output_tools = Some(PathBuf::from(value));
            }
            "--context" => {
                let text = read_text(Path::new(value), 16 * 1024)?;
                let context = eal_core::json::parse(&text, &eal_core::ExecutionLimits::default())?;
                if !context.is_object() {
                    return Err("Package context must be a JSON object".into());
                }
                request["context"] = context;
            }
            "--claim" => request["claim"] = json!(value),
            "--now" => {
                if mode != "run" {
                    return Err("--now belongs to run".into());
                }
                eal_core::validation::parse_time(value)?;
                request["now"] = json!(value);
            }
            "--tools" => {
                let path = Path::new(value)
                    .canonicalize()
                    .map_err(|error| error.to_string())?;
                settings.extend([
                    "--registry".into(),
                    path.to_str().ok_or("Tool path must be UTF-8")?.into(),
                ]);
            }
            _ => settings.extend([flag.into(), value.into()]),
        }
        index += 1;
    }
    if !seen.contains("--tools") || !seen.contains("--workspace") {
        return Err("run/prepare requires --tools and --workspace".into());
    }
    if !seen.contains("--context") {
        let manifest_index = settings
            .iter()
            .position(|arg| arg == "--registry")
            .ok_or("Missing tools manifest")?;
        request["context"] =
            eal_runtime::acquisition::ToolRegistry::load(&settings[manifest_index + 1])
                .map_err(|error| error.to_string())?
                .context;
    }
    let workspace_index = settings
        .iter()
        .position(|arg| arg == "--workspace")
        .ok_or("Missing run workspace")?;
    std::fs::create_dir_all(&settings[workspace_index + 1]).map_err(|error| error.to_string())?;
    let encoded = eal_core::canonical::canonical_json_bytes(
        &request,
        &eal_core::ExecutionLimits {
            record_bytes: super::MAX_REQUEST_BYTES,
            ..Default::default()
        },
    )?;
    if encoded.len() > super::MAX_REQUEST_BYTES {
        return Err("Package request exceeds host limit".into());
    }
    let output_tools = output_tools.map(|output| -> Result<(PathBuf, PathBuf), String> {
        let manifest_index = settings.iter().position(|arg| arg == "--registry")
            .ok_or("Missing tools manifest")?;
        let input = PathBuf::from(&settings[manifest_index + 1]);
        let parent = output.parent().filter(|path| !path.as_os_str().is_empty()).unwrap_or(Path::new("."))
            .canonicalize().map_err(|error| error.to_string())?;
        if input.parent() != Some(parent.as_path()) {
            return Err("Prepared TOML must be written beside the input manifest so relative resources retain their authority".into());
        }
        let output = parent.join(output.file_name().ok_or("Output tools path has no filename")?);
        if output.exists() || output.symlink_metadata().is_ok() {
            return Err("Prepared tools output must be a new file".into());
        }
        Ok((input, output))
    }).transpose()?;
    Ok(PackageInvocation {
        args: settings,
        request: Some(String::from_utf8(encoded).map_err(|error| error.to_string())?),
        output_tools,
    })
}

fn read_text(path: &Path, limit: usize) -> Result<String, String> {
    let path = path.canonicalize().map_err(|error| error.to_string())?;
    let bytes = eal_runtime::acquisition::read_regular(
        &path,
        limit,
        "Package input must be a regular file",
    )
    .map_err(|error| error.to_string())?;
    String::from_utf8(bytes).map_err(|_| "Package input must be UTF-8".into())
}

/// Install only generated identity fields into the original operator manifest.
/// Paths, commands, credentials, mappings and limits remain operator-owned.
pub(super) fn publish_prepared_tools(
    input: &Path,
    output: &Path,
    response: &mut serde_json::Value,
) -> Result<(), String> {
    use sha2::{Digest, Sha256};
    if response["is_error"] == true {
        return Ok(());
    }
    let receipt = &response["result"];
    if receipt["schema"] != "EAL/preparation/1" {
        return Err("Preparation returned an unsupported receipt".into());
    }
    let bytes = eal_runtime::acquisition::read_regular(
        input,
        1_048_576,
        "Tool manifest must be a regular file",
    )
    .map_err(|error| error.to_string())?;
    if receipt["manifest_sha256"].as_str() != Some(&format!("{:x}", Sha256::digest(&bytes))) {
        return Err("Tool manifest changed after preparation".into());
    }
    let mut document = eal_collector_contracts::RegistryDocument::from_toml(
        std::str::from_utf8(&bytes).map_err(|_| "Tool manifest must be UTF-8")?,
    )?;
    for (name, prepared) in receipt["tools"]
        .as_object()
        .ok_or("Preparation omitted tool identities")?
    {
        let binding = document
            .tools
            .get_mut(name)
            .ok_or("Preparation names an unconfigured tool")?;
        if prepared["kind"] != binding.kind.as_str() || prepared["version"] != binding.version {
            return Err("Prepared tool differs from operator binding".into());
        }
        if let Some(pins) = prepared.get("pins") {
            let direct = binding
                .direct
                .as_mut()
                .ok_or("Prepared tool omitted direct configuration")?;
            if prepared["interface"]["tool"].as_str() != direct.tool.as_deref() {
                return Err("Prepared interface names a different operation".into());
            }
            direct.input_schema_sha256 =
                serde_json::from_value(pins["input_schema_sha256"].clone())
                    .map_err(|error| error.to_string())?;
            direct.output_schema_sha256 =
                serde_json::from_value(pins["output_schema_sha256"].clone())
                    .map_err(|error| error.to_string())?;
            direct.server_identity = serde_json::from_value(pins["server_identity"].clone())
                .map_err(|error| error.to_string())?;
            direct.validate(binding.kind)?;
        }
    }
    let text = toml::to_string(&document).map_err(|error| error.to_string())?;
    if text.len() > 1_048_576 {
        return Err("Prepared manifest exceeds the tool manifest allowance".into());
    }
    publish_file(output, text.as_bytes())?;
    response["result"]["output_tools"] = json!(output);
    Ok(())
}

/// Publish a complete private file without replacing an existing destination.
fn publish_file(output: &Path, bytes: &[u8]) -> Result<(), String> {
    use std::{
        io::Write,
        sync::atomic::{AtomicU64, Ordering},
    };
    static SEQUENCE: AtomicU64 = AtomicU64::new(0);
    let parent = output
        .parent()
        .ok_or("Prepared manifest has no parent directory")?;
    let staged = parent.join(format!(
        ".eal-prepared-{}-{}",
        std::process::id(),
        SEQUENCE.fetch_add(1, Ordering::Relaxed)
    ));
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(&staged).map_err(|error| error.to_string())?;
    let result = file
        .write_all(bytes)
        .and_then(|()| file.sync_all())
        .and_then(|()| std::fs::hard_link(&staged, output));
    drop(file);
    let cleanup = std::fs::remove_file(&staged);
    result.map_err(|error| error.to_string())?;
    cleanup.map_err(|error| error.to_string())?;
    Ok(())
}
