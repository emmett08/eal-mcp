//! Bounded subprocess execution independent of EAL language, storage and adapters.
#![forbid(unsafe_code)]
pub mod duplex;
pub mod process;
pub mod supervisor;
pub use process::*;

/// Resolve the sole installed companion name. A missing executable fails at
/// launch; no compatibility alias or unsupervised fallback is selected.
pub fn companion_in(directory: &std::path::Path) -> std::path::PathBuf {
    directory.join(format!(
        "eal-process-supervisor{}",
        std::env::consts::EXE_SUFFIX
    ))
}
