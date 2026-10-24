//! Bounded subprocess execution independent of EAL language, storage and adapters.
#![forbid(unsafe_code)]
pub mod process;
pub mod supervisor;
pub use process::*;

/// Preserve the EAL 0.1 installed alias when present during the compatibility window.
/// New installations may provide only the canonical narrow-package companion.
/// The returned missing canonical path fails explicitly at launch.
pub fn companion_in(directory: &std::path::Path) -> std::path::PathBuf {
    let legacy = directory.join(format!(
        "eal-command-supervisor{}",
        std::env::consts::EXE_SUFFIX
    ));
    if legacy.is_file() {
        return legacy;
    }
    directory.join(format!(
        "eal-process-supervisor{}",
        std::env::consts::EXE_SUFFIX
    ))
}
