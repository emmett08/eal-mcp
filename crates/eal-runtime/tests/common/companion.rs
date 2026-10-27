//! Resolve the separately built canonical process companion for integration tests.
pub fn path() -> std::path::PathBuf {
    let worker = std::path::Path::new(env!("CARGO_BIN_EXE_eal-method-worker"));
    let path = eal_process::companion_in(worker.parent().expect("profile directory"));
    assert!(
        path.is_file(),
        "Build eal-process binaries before focused runtime tests"
    );
    path
}
