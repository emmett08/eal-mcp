//! Read bounded UTF-8 inputs from one operator-selected workspace.
use eal_core::ExecutionLimits;
use serde::Deserialize;
use serde_json::Value;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

pub struct Workspace {
    root: PathBuf,
    #[cfg(unix)]
    directory: std::fs::File,
}

impl Workspace {
    pub fn new(root: &Path) -> Result<Self, String> {
        let root = root.canonicalize().map_err(|error| error.to_string())?;
        if !root.is_dir() {
            return Err("Workspace must be a directory".into());
        }
        #[cfg(unix)]
        let directory = std::fs::File::from(
            rustix::fs::open(
                &root,
                rustix::fs::OFlags::RDONLY
                    | rustix::fs::OFlags::DIRECTORY
                    | rustix::fs::OFlags::NOFOLLOW
                    | rustix::fs::OFlags::CLOEXEC,
                rustix::fs::Mode::empty(),
            )
            .map_err(|error| error.to_string())?,
        );
        Ok(Self {
            root,
            #[cfg(unix)]
            directory,
        })
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

    pub fn output_path(&self, name: &str) -> Result<PathBuf, String> {
        let requested = self.root.join(name);
        if std::fs::symlink_metadata(&requested).is_ok() {
            return self.path(name);
        }
        let parent = requested
            .parent()
            .ok_or("Output must have a parent directory")?
            .canonicalize()
            .map_err(|error| error.to_string())?;
        if !parent.starts_with(&self.root) {
            return Err("Path escapes the workspace".into());
        }
        let filename = requested.file_name().ok_or("Output must name a file")?;
        Ok(parent.join(filename))
    }

    pub fn write_output(&self, name: &str, bytes: &[u8]) -> Result<PathBuf, String> {
        let path = self.output_path(name)?;
        #[cfg(unix)]
        let mut file = {
            use rustix::fs::{Mode, OFlags, openat};
            let relative = path
                .strip_prefix(&self.root)
                .map_err(|_| "Path escapes the workspace")?;
            let mut directory = self
                .directory
                .try_clone()
                .map_err(|error| error.to_string())?;
            let parent = relative
                .parent()
                .ok_or("Output must have a parent directory")?;
            for component in parent.components() {
                let std::path::Component::Normal(component) = component else {
                    return Err("Output path must stay within the workspace".into());
                };
                directory = std::fs::File::from(
                    openat(
                        &directory,
                        component,
                        OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
                        Mode::empty(),
                    )
                    .map_err(|error| error.to_string())?,
                );
            }
            std::fs::File::from(
                openat(
                    &directory,
                    relative.file_name().ok_or("Output must name a file")?,
                    OFlags::WRONLY
                        | OFlags::CREATE
                        | OFlags::NOFOLLOW
                        | OFlags::NONBLOCK
                        | OFlags::CLOEXEC,
                    Mode::from_raw_mode(0o666),
                )
                .map_err(|error| error.to_string())?,
            )
        };
        #[cfg(not(unix))]
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(false)
            .open(&path)
            .map_err(|error| error.to_string())?;
        if !file
            .metadata()
            .map_err(|error| error.to_string())?
            .is_file()
        {
            return Err("Output must be a regular file".into());
        }
        file.set_len(0).map_err(|error| error.to_string())?;
        file.write_all(bytes).map_err(|error| error.to_string())?;
        Ok(path)
    }

    pub fn read(&self, name: &str, maximum: usize) -> Result<String, String> {
        let path = self.path(name)?;
        self.read_path(&path, maximum)
    }

    fn read_path(&self, path: &Path, maximum: usize) -> Result<String, String> {
        // Resolve links first, then open every canonical component from the
        // retained workspace descriptor so a replacement cannot redirect it.
        #[cfg(unix)]
        let input = {
            use rustix::fs::{Mode, OFlags, openat};
            let relative = path
                .strip_prefix(&self.root)
                .map_err(|_| "Path escapes the workspace")?;
            let mut directory = self
                .directory
                .try_clone()
                .map_err(|error| error.to_string())?;
            let parent = relative.parent().ok_or("Input must name a file")?;
            for component in parent.components() {
                let std::path::Component::Normal(component) = component else {
                    return Err("Input path must stay within the workspace".into());
                };
                directory = std::fs::File::from(
                    openat(
                        &directory,
                        component,
                        OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
                        Mode::empty(),
                    )
                    .map_err(|error| error.to_string())?,
                );
            }
            std::fs::File::from(
                openat(
                    &directory,
                    relative.file_name().ok_or("Input must name a file")?,
                    OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC,
                    Mode::empty(),
                )
                .map_err(|error| error.to_string())?,
            )
        };
        #[cfg(not(unix))]
        let input = std::fs::File::open(path).map_err(|error| error.to_string())?;
        if !input
            .metadata()
            .map_err(|error| error.to_string())?
            .is_file()
        {
            return Err("Input must be a regular file".into());
        }
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
        self.read_path(&path, maximum)
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

#[cfg(all(test, unix))]
mod tests {
    use super::Workspace;
    use std::os::unix::fs::symlink;

    #[test]
    fn contained_links_are_read_but_replaced_components_are_not_followed() {
        let base = std::env::temp_dir().join(format!(
            "eal-input-read-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let root = base.join("workspace");
        let outside = base.join("outside");
        std::fs::create_dir_all(root.join("nested")).unwrap();
        std::fs::create_dir(&outside).unwrap();
        std::fs::write(root.join("nested/library.eal"), "inside").unwrap();
        std::fs::write(outside.join("library.eal"), "outside").unwrap();
        symlink("nested", root.join("alias")).unwrap();
        let workspace = Workspace::new(&root).unwrap();
        assert_eq!(
            workspace.imported_source("alias/library.eal", 6).unwrap(),
            "inside"
        );
        let resolved = workspace.path("alias/library.eal").unwrap();
        std::fs::rename(root.join("nested"), root.join("original")).unwrap();
        symlink(&outside, root.join("nested")).unwrap();
        assert!(workspace.read_path(&resolved, 20).is_err());
        std::fs::remove_dir_all(base).unwrap();
    }
}
