//! Read bounded UTF-8 inputs from one operator-selected workspace.
use eal_core::ExecutionLimits;
use serde::Deserialize;
use serde_json::Value;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

pub struct Workspace {
    root: PathBuf,
}

impl Workspace {
    pub fn new(root: &Path) -> Result<Self, String> {
        let root = root.canonicalize().map_err(|error| error.to_string())?;
        if !root.is_dir() {
            return Err("Workspace must be a directory".into());
        }
        Ok(Self { root })
    }

    fn path(&self, name: &str) -> Result<PathBuf, String> {
        let path = self
            .root
            .join(name)
            .canonicalize()
            .map_err(|error| format!("Cannot read {name:?}: {error}"))?;
        if !path.starts_with(&self.root) {
            return Err("Path escapes the workspace".into());
        }
        if !path.is_file() {
            return Err("Input must be a regular file".into());
        }
        Ok(path)
    }

    pub fn origin(&self, name: &str) -> Result<String, String> {
        if name == "-" {
            return Ok(String::new());
        }
        let path = self.path(name)?;
        let relative = path
            .strip_prefix(&self.root)
            .map_err(|_| "Path escapes the workspace")?;
        relative
            .to_str()
            .map(|path| path.replace('\\', "/"))
            .ok_or_else(|| "Source path must be UTF-8".into())
    }

    pub fn read(&self, name: &str, maximum: usize) -> Result<String, String> {
        let path = self.path(name)?;
        let input = std::fs::File::open(path).map_err(|error| error.to_string())?;
        read_bounded(input, maximum)
    }

    pub fn source(&self, name: &str, maximum: usize) -> Result<String, String> {
        if name == "-" {
            read_bounded(io::stdin(), maximum)
        } else {
            self.read(name, maximum)
        }
    }

    pub fn imported_source(&self, name: &str, maximum: usize) -> Result<String, String> {
        let path = self.path(name)?;
        if path.extension() != Some(std::ffi::OsStr::new("eal")) {
            return Err("An import must name a workspace .eal file".into());
        }
        self.read(name, maximum)
    }

    pub fn json_file(&self, name: &str, limits: &ExecutionLimits) -> Result<Value, String> {
        eal_core::json::parse(&self.read(name, limits.record_bytes)?, limits)
    }

    pub fn context(&self, value: Option<&str>, limits: &ExecutionLimits) -> Result<Value, String> {
        let value = match value {
            Some(value) if value.starts_with('@') => self.json_file(&value[1..], limits)?,
            Some(value) => eal_core::json::parse(value, limits)?,
            None => serde_json::json!({}),
        };
        if !value.is_object() {
            return Err("Context must be a JSON object".into());
        }
        Ok(value)
    }

    pub fn limits(&self, name: Option<&str>) -> Result<ExecutionLimits, String> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Document {
            limits: ExecutionLimits,
        }
        match name {
            Some(name) => {
                let document: Document = toml::from_str(&self.read(name, 16 * 1024)?)
                    .map_err(|error| error.to_string())?;
                Ok(document.limits)
            }
            None => Ok(ExecutionLimits::default()),
        }
    }
}

fn read_bounded(reader: impl Read, maximum: usize) -> Result<String, String> {
    let mut bytes = Vec::new();
    reader
        .take((maximum as u64).saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(|error| error.to_string())?;
    if bytes.len() > maximum {
        return Err(format!("Input exceeds its {maximum} byte budget"));
    }
    String::from_utf8(bytes).map_err(|error| error.to_string())
}
