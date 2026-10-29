//! Admit bounded files, exact consumer identity and operator command options.
use super::{CONSUMER_BLOB, CONSUMER_SCHEMA_BLOB, MAX_BYTES, MAX_INPUT_BYTES, Result};
use serde_json::Value;
use sha1::Sha1;
use sha2::Digest;
use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};
#[derive(Default)]
pub(super) struct Options {
    pub(super) consumer: PathBuf,
    pub(super) binary: PathBuf,
    pub(super) node: Option<PathBuf>,
    pub(super) views: Option<PathBuf>,
    pub(super) save_views: Option<PathBuf>,
    pub(super) root: Option<PathBuf>,
    pub(super) supervisor: Option<PathBuf>,
}
impl Options {
    pub(super) fn parse(args: &[String]) -> Result<Self> {
        let mut options = Self::default();
        let mut args = args.iter();
        let mut positional = Vec::new();
        let mut flags = BTreeSet::new();
        while let Some(flag) = args.next() {
            if !flag.starts_with('-') {
                positional.push(PathBuf::from(flag));
                continue;
            }
            if !flags.insert(flag) {
                return Err(format!("Duplicate argument {flag}"));
            }
            let value = PathBuf::from(
                args.next()
                    .ok_or_else(|| format!("Missing value for {flag}"))?,
            );
            match flag.as_str() {
                "--consumer" => options.consumer = value,
                "--binary" => options.binary = value,
                "--node" => options.node = Some(value),
                "--views" => options.views = Some(value),
                "--save-views" => options.save_views = Some(value),
                "--root" => options.root = Some(value),
                "--supervisor" => options.supervisor = Some(value),
                _ => return Err(format!("Unknown consumer argument {flag}")),
            }
        }
        if !positional.is_empty() {
            if positional.len() != 2
                || !options.consumer.as_os_str().is_empty()
                || !options.binary.as_os_str().is_empty()
            {
                return Err("Supply --consumer and --binary, or two positional paths".into());
            }
            options.consumer = positional.remove(0);
            options.binary = positional.remove(0);
        }
        if options.consumer.as_os_str().is_empty() || options.binary.as_os_str().is_empty() {
            return Err("Consumer check requires --consumer FILE and --binary FILE".into());
        }
        Ok(options)
    }
}

pub(super) struct Temporary(pub(super) PathBuf);
impl Temporary {
    pub(super) fn new() -> Result<Self> {
        let path =
            std::env::temp_dir().join(format!("eal-consumer-check-{}", uuid::Uuid::new_v4()));
        fs::create_dir(&path).map_err(|e| e.to_string())?;
        Ok(Self(path))
    }
}
impl Drop for Temporary {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

pub(super) fn read_bytes(path: &Path, maximum: usize) -> Result<Vec<u8>> {
    crate::smoke::read_bytes(path, maximum)
}
pub(super) fn parse_json(bytes: &[u8], maximum: usize) -> Result<Value> {
    let text = std::str::from_utf8(bytes).map_err(|e| e.to_string())?;
    let limits = eal_core::ExecutionLimits {
        record_bytes: maximum,
        json_nodes: 4_000_000,
        json_depth: 128,
        ..Default::default()
    };
    eal_core::json::parse(text, &limits)
}
pub(super) fn load(path: &Path) -> Result<Value> {
    parse_json(&read_bytes(path, MAX_INPUT_BYTES)?, MAX_INPUT_BYTES)
}
pub(super) fn required_string<'a>(value: &'a Value, key: &str) -> Result<&'a str> {
    value
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("Fixture field {key} must be a string"))
}
pub(super) fn array<'a>(value: &'a Value, key: &str) -> Result<&'a Vec<Value>> {
    value
        .get(key)
        .and_then(Value::as_array)
        .ok_or_else(|| format!("Field {key} must be an array"))
}

pub(super) fn git_blob(bytes: &[u8]) -> String {
    let mut digest = Sha1::new();
    digest.update(format!("blob {}\0", bytes.len()).as_bytes());
    digest.update(bytes);
    format!("{:x}", digest.finalize())
}
pub(super) fn checked_consumer(path: &Path) -> Result<String> {
    let content = read_bytes(path, MAX_BYTES)?;
    let blob = git_blob(&content);
    if blob != CONSUMER_BLOB {
        return Err(format!(
            "Expected pinned consumer blob {CONSUMER_BLOB}, found {blob}"
        ));
    }
    let source = String::from_utf8(content).map_err(|e| e.to_string())?;
    let root = path
        .canonicalize()
        .map_err(|e| e.to_string())?
        .ancestors()
        .nth(3)
        .ok_or("Consumer must be src/domain/aspic.ts within its checkout")?
        .to_owned();
    let schema = root.join("schema/aspic-view.schema.json");
    let schema_bytes = read_bytes(&schema, MAX_BYTES)?;
    let schema_blob = git_blob(&schema_bytes);
    if schema_blob != CONSUMER_SCHEMA_BLOB {
        return Err(format!(
            "Expected pinned consumer schema blob {CONSUMER_SCHEMA_BLOB}, found {schema_blob}"
        ));
    }
    let ajv = root.join("node_modules/ajv/dist/2020.js");
    if !ajv.is_file() {
        return Err("Consumer Ajv dependency is unavailable; run npm ci in its checkout".into());
    }
    relocated_importer(&source, &ajv, &schema)
}

/// Node strips TypeScript but requires explicit JSON import attributes. Relocate
/// only imports; the complete pinned importer and all its checks stay unchanged.
fn relocated_importer(source: &str, ajv: &Path, schema: &Path) -> Result<String> {
    let replacements = [
        (
            "import Ajv2020 from 'ajv/dist/2020.js'",
            format!(
                "import Ajv2020 from {}",
                serde_json::to_string(ajv).map_err(|e| e.to_string())?
            ),
        ),
        (
            "import schema from '../../schema/aspic-view.schema.json'",
            format!(
                "import schema from {} with {{ type: 'json' }}",
                serde_json::to_string(schema).map_err(|e| e.to_string())?
            ),
        ),
    ];
    let mut relocated = source.to_owned();
    for (original, replacement) in replacements {
        if relocated.matches(original).count() != 1 {
            return Err("Pinned consumer import declaration is missing or duplicated".into());
        }
        relocated = relocated.replacen(original, &replacement, 1);
    }
    Ok(relocated)
}

pub(super) fn executable(path: &Path) -> Result<PathBuf> {
    let path = if path.components().count() == 1 {
        std::env::var_os("PATH")
            .into_iter()
            .flat_map(|s| std::env::split_paths(&s).collect::<Vec<_>>())
            .map(|directory| directory.join(path))
            .find(|p| p.is_file())
            .ok_or_else(|| format!("Executable {} was not found", path.display()))?
    } else {
        path.to_owned()
    };
    let path = path.canonicalize().map_err(|e| e.to_string())?;
    if !path.is_file() {
        return Err(format!("Expected executable file {}", path.display()));
    }
    Ok(path)
}
