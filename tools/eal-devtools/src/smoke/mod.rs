//! Installed native application, coordination and consumer acceptance checks.
mod applications;
mod consumer;
mod coordination;
pub mod harness;
mod operation_contracts;
mod process;

use serde_json::{Value, json};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

pub type Result<T> = std::result::Result<T, String>;
pub fn ensure(condition: bool, message: impl Into<String>) -> Result<()> {
    if condition {
        Ok(())
    } else {
        Err(message.into())
    }
}
pub(super) fn read_bytes(path: &Path, maximum: usize) -> Result<Vec<u8>> {
    let absolute = std::path::absolute(path).map_err(|e| e.to_string())?;
    let name = absolute
        .file_name()
        .ok_or("Smoke input requires a file name")?;
    let parent = absolute
        .parent()
        .ok_or("Smoke input requires a parent")?
        .canonicalize()
        .map_err(|e| e.to_string())?;
    let selected = parent.join(name);
    let metadata = std::fs::symlink_metadata(&selected).map_err(|e| e.to_string())?;
    ensure(
        metadata.is_file(),
        "Smoke input requires a regular file without a leaf alias",
    )?;
    ensure(
        metadata.len() <= maximum as u64,
        "Smoke input exceeds its byte allowance",
    )?;
    let bytes = eal_runtime::acquisition::read_regular(
        &selected,
        maximum,
        "Smoke input requires a regular file",
    )
    .map_err(|error| error.message)?;
    ensure(
        bytes.len() <= maximum,
        "Smoke input exceeds its byte allowance",
    )?;
    Ok(bytes)
}
pub fn read_json(path: &Path) -> Result<Value> {
    parse_json(&read_bytes(path, 64 * 1024 * 1024)?, 64 * 1024 * 1024)
}
pub fn parse_json(bytes: &[u8], maximum: usize) -> Result<Value> {
    ensure(
        bytes.len() <= maximum,
        "Smoke JSON exceeds its byte allowance",
    )?;
    let text = std::str::from_utf8(bytes).map_err(|e| e.to_string())?;
    eal_core::json::parse(
        text,
        &eal_core::ExecutionLimits {
            record_bytes: maximum,
            json_nodes: 4_000_000,
            json_depth: 128,
            ..Default::default()
        },
    )
}
pub fn write_json(path: &Path, value: &Value) -> Result<()> {
    let mut bytes = serde_json::to_vec_pretty(value).map_err(|e| e.to_string())?;
    bytes.push(b'\n');
    std::fs::write(path, bytes).map_err(|e| e.to_string())
}
#[derive(Debug)]
pub struct Options {
    pub family: String,
    pub prefix: PathBuf,
    pub workspace: Option<PathBuf>,
    pub worker: Option<PathBuf>,
    pub consumer: Option<PathBuf>,
    pub binary: Option<PathBuf>,
    pub node: Option<PathBuf>,
    pub views: Option<PathBuf>,
    pub save_views: Option<PathBuf>,
    pub root: Option<PathBuf>,
    pub supervisor: Option<PathBuf>,
}
impl Options {
    pub fn parse(arguments: impl IntoIterator<Item = String>) -> Result<Self> {
        let mut args = arguments.into_iter();
        let family = args.next().ok_or("Usage: eal-smoke applications|coordination|consumer --prefix DIR [--workspace DIR] [--worker FILE]")?;
        if matches!(family.as_str(), "--help" | "-h") {
            return Err("Usage: eal-smoke applications|coordination|consumer --prefix DIR [--workspace DIR] [--worker FILE]\nconsumer options: --consumer FILE --binary FILE --node FILE [--views FILE] [--save-views FILE]".into());
        }
        let mut options = Self {
            family,
            prefix: PathBuf::new(),
            workspace: None,
            worker: None,
            consumer: None,
            binary: None,
            node: None,
            views: None,
            save_views: None,
            root: None,
            supervisor: None,
        };
        let mut flags = BTreeSet::new();
        while let Some(flag) = args.next() {
            ensure(
                flags.insert(flag.clone()),
                format!("Duplicate argument {flag}"),
            )?;
            let allowed: &[&str] = match options.family.as_str() {
                "applications" => &["--prefix", "--workspace"],
                "coordination" => &["--prefix", "--workspace", "--worker"],
                "consumer" => &[
                    "--consumer",
                    "--binary",
                    "--node",
                    "--views",
                    "--save-views",
                    "--root",
                    "--supervisor",
                ],
                _ => return Err(format!("Unknown smoke family {}", options.family)),
            };
            ensure(
                allowed.contains(&flag.as_str()),
                format!("Unknown {} argument {flag}", options.family),
            )?;
            let value = PathBuf::from(
                args.next()
                    .ok_or_else(|| format!("Missing value for {flag}"))?,
            );
            match flag.as_str() {
                "--prefix" => options.prefix = value,
                "--workspace" => options.workspace = Some(value),
                "--worker" => options.worker = Some(value),
                "--consumer" => options.consumer = Some(value),
                "--binary" => options.binary = Some(value),
                "--node" => options.node = Some(value),
                "--views" => options.views = Some(value),
                "--save-views" => options.save_views = Some(value),
                "--root" => options.root = Some(value),
                "--supervisor" => options.supervisor = Some(value),
                _ => return Err(format!("Unknown argument {flag}")),
            }
        }
        if matches!(options.family.as_str(), "applications" | "coordination") {
            ensure(
                !options.prefix.as_os_str().is_empty(),
                "Installed acceptance requires --prefix DIR",
            )?;
        }
        if options.family == "coordination" {
            ensure(
                options.worker.is_some(),
                "Coordination acceptance requires --worker FILE",
            )?;
        }
        Ok(options)
    }
}
pub fn run(options: Options) -> Result<Value> {
    match options.family.as_str() {
        "applications" => applications::run(&options),
        "coordination" => coordination::run(&options),
        "consumer" => {
            let mut args = Vec::new();
            for (flag, value) in [
                ("--consumer", &options.consumer),
                ("--binary", &options.binary),
                ("--node", &options.node),
                ("--views", &options.views),
                ("--save-views", &options.save_views),
                ("--root", &options.root),
                ("--supervisor", &options.supervisor),
            ] {
                if let Some(value) = value {
                    args.extend([flag.into(), value.display().to_string()]);
                }
            }
            consumer::run(&args)
        }
        _ => Err(format!("Unknown smoke family {}", options.family)),
    }
}
pub fn result_error(message: &str) -> Value {
    json!({"error":message})
}

#[cfg(test)]
mod tests {
    use super::*;
    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|v| v.to_string()).collect()
    }
    #[test]
    fn admission_rejects_duplicate_unknown_and_missing_arguments() {
        assert!(parse_json(br#"{"key":1,"key":2}"#, 1024).is_err());
        for values in [
            vec!["applications", "--prefix", "a", "--prefix", "b"],
            vec!["applications", "--unknown", "x"],
            vec!["applications", "--prefix"],
        ] {
            assert!(Options::parse(args(&values)).is_err());
        }
        let parsed = Options::parse(args(&[
            "consumer",
            "--consumer",
            "graph.ts",
            "--binary",
            "eal-core",
            "--root",
            "repo",
            "--supervisor",
            "supervisor",
        ]))
        .unwrap();
        assert_eq!(parsed.consumer, Some(PathBuf::from("graph.ts")));
        assert_eq!(parsed.supervisor, Some(PathBuf::from("supervisor")));
    }
    #[test]
    fn existing_nonempty_workspace_is_preserved() {
        let root =
            std::env::temp_dir().join(format!("eal-smoke-admission-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&root).unwrap();
        std::fs::write(root.join("existing"), b"retained").unwrap();
        let prefix = std::env::current_exe()
            .unwrap()
            .parent()
            .unwrap()
            .to_owned();
        let options = Options::parse(args(&[
            "applications",
            "--prefix",
            prefix.to_str().unwrap(),
            "--workspace",
            root.to_str().unwrap(),
        ]))
        .unwrap();
        assert!(harness::Harness::new(&options, "all", &[]).is_err());
        assert_eq!(std::fs::read(root.join("existing")).unwrap(), b"retained");
        assert_eq!(std::fs::read_dir(&root).unwrap().count(), 1);
        std::fs::remove_dir_all(root).unwrap();
    }
}
