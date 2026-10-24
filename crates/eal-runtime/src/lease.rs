//! Descriptor-owned local admission; the last inherited descriptor releases it.
use crate::store::files;
#[cfg(unix)]
use std::os::fd::{AsFd, BorrowedFd};
use std::{
    fs::File,
    path::{Path, PathBuf},
    sync::{Arc, OnceLock},
    time::Duration,
};

#[derive(Clone, Debug)]
pub struct WorkspaceAcquisition {
    path: PathBuf,
    directory: files::Identity,
    lock: Arc<OnceLock<files::Identity>>,
}
impl WorkspaceAcquisition {
    pub fn new(workspace: impl AsRef<Path>) -> Result<Self, String> {
        let path = files::absolute(&workspace.as_ref().join(".eal/mcp-acquisition.lock"))?;
        let directory =
            files::create_directory(path.parent().unwrap(), "The MCP acquisition directory")?;
        if std::fs::symlink_metadata(&path).is_ok_and(|metadata| metadata.file_type().is_symlink())
        {
            return Err("The MCP acquisition lock must not be a symlink".into());
        }
        Ok(Self {
            path,
            directory,
            lock: Arc::new(OnceLock::new()),
        })
    }
    pub fn path(&self) -> &Path {
        &self.path
    }
    pub fn hold(&self, timeout: Duration) -> Result<AcquisitionLease, String> {
        files::check_directory(
            self.path.parent().unwrap(),
            self.directory,
            "The MCP acquisition directory",
        )?;
        let (file, identity) = files::open_private(
            &self.path,
            true,
            true,
            self.lock.get().copied(),
            "The MCP acquisition lock",
        )?;
        files::lock(&file, timeout, "the MCP acquisition lock")?;
        files::check_directory(
            self.path.parent().unwrap(),
            self.directory,
            "The MCP acquisition directory",
        )?;
        files::validate_file(
            &self.path,
            &file,
            Some(identity),
            "The MCP acquisition lock",
        )?;
        if *self.lock.get_or_init(|| identity) != identity {
            return Err("The MCP acquisition lock was replaced".into());
        }
        Ok(AcquisitionLease { file })
    }
}
/// Closing this copy never explicitly unlocks another inherited copy of the
/// same open-file description. Supervisors receive a borrowed FD explicitly.
#[derive(Debug)]
pub struct AcquisitionLease {
    file: File,
}
#[cfg(unix)]
impl AsFd for AcquisitionLease {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.file.as_fd()
    }
}
#[cfg(windows)]
impl std::os::windows::io::AsHandle for AcquisitionLease {
    fn as_handle(&self) -> std::os::windows::io::BorrowedHandle<'_> {
        std::os::windows::io::AsHandle::as_handle(&self.file)
    }
}

impl eal_process::Retention for AcquisitionLease {
    fn duplicate_for_supervisor(&self) -> std::io::Result<std::process::Stdio> {
        Ok(std::process::Stdio::from(self.file.try_clone()?))
    }
}
