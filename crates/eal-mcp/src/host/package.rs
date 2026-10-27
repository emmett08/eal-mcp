//! Translate a portable source package into the ordinary MCP execute request.
use serde_json::json;
use std::{collections::BTreeSet, path::Path};

/// No alternate interpreter: this only reads bounded files and builds the same
/// closed operation request accepted from a text model or native MCP client.
pub(super) fn prepare(argv: Vec<String>) -> Result<(Vec<String>, Option<String>), String> {
    if argv.first().map(String::as_str) != Some("run") {
        return Ok((argv, None));
    }
    let source_path = argv
        .get(1)
        .filter(|path| !path.starts_with('-'))
        .ok_or("run requires an EAL source file")?;
    let source = read_text(Path::new(source_path), super::MAX_REQUEST_BYTES)?;
    let mut settings = Vec::new();
    let mut request = json!({"operation":"execute","source":source,"context":{}});
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
        return Err("run requires --tools and --workspace".into());
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
    Ok((
        settings,
        Some(String::from_utf8(encoded).map_err(|error| error.to_string())?),
    ))
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
