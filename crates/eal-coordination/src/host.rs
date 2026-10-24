//! Operator-selected installed native executables, separate from frozen task data.
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub struct HostPaths {
    pub eal_mcp: PathBuf,
    pub eal_coordinate: PathBuf,
    pub supervisor: PathBuf,
    pub method_worker: PathBuf,
}
impl HostPaths {
    pub fn validate(&self) -> Result<(), String> {
        for path in [
            &self.eal_mcp,
            &self.eal_coordinate,
            &self.supervisor,
            &self.method_worker,
        ] {
            if !path.is_absolute() || path.to_str().is_none() {
                return Err("Native host executable paths must be absolute UTF-8 paths".into());
            }
        }
        Ok(())
    }
    pub fn from_directory(directory: &Path) -> Self {
        let executable =
            |name: &str| directory.join(format!("{name}{}", std::env::consts::EXE_SUFFIX));
        Self {
            eal_mcp: executable("eal-mcp"),
            eal_coordinate: executable("eal-coordinate"),
            supervisor: eal_runtime::bootstrap::supervisor_in(directory),
            method_worker: executable("eal-method-worker"),
        }
    }
    pub fn current() -> Result<Self, String> {
        let executable = std::env::current_exe().map_err(|e| e.to_string())?;
        Ok(Self::from_directory(
            executable
                .parent()
                .ok_or("Cannot locate native executable directory")?,
        ))
    }
}
