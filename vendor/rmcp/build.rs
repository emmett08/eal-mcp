//! Reconstruct the pinned SDK source without running upstream's git-hook installer.
#![forbid(unsafe_code)]
use flate2::read::GzDecoder;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    env, fs,
    io::{self, Read, Write},
    path::{Component, Path, PathBuf},
};

const ARCHIVE: &[u8] = include_bytes!("rmcp-3.6.0.crate");
const ARCHIVE_SHA256: &str = "ceb42e9e47443cd83a834dea57b5816d35481417bae72ade5dc5615c59122e26";
const ARCHIVE_BYTES: usize = 624_559;
const SOURCE_ROOT: &str = "rmcp-3.6.0";
const ROOT_HEADER: &str = "#![cfg_attr(docsrs, feature(doc_cfg))]\n#![cfg_attr(docsrs, allow(unused_attributes))]\n#![doc = include_str!(\"../README.md\")]\n";

/// A hunk is applied only when its complete before-text occurs exactly once.
struct Patch {
    target: &'static str,
    before: &'static str,
    after: &'static str,
}
include!("patches.rs");
struct AddedFile {
    target: &'static str,
    contents: &'static str,
}
include!("additions.rs");

fn main() {
    if let Err(error) = reconstruct() {
        panic!("Pinned rmcp reconstruction failed: {error}");
    }
}

fn reconstruct() -> io::Result<()> {
    for input in [
        "build.rs",
        "src/lib.rs",
        "rmcp-3.6.0.crate",
        "patches.rs",
        "additions.rs",
        "patches",
        "UPSTREAM.json",
    ] {
        println!("cargo:rerun-if-changed={input}");
    }
    ensure(
        ARCHIVE.len() == ARCHIVE_BYTES,
        "official SDK archive byte count differs",
    )?;
    ensure(
        format!("{:x}", Sha256::digest(ARCHIVE)) == ARCHIVE_SHA256,
        "official SDK archive SHA-256 differs",
    )?;
    let output = PathBuf::from(
        env::var_os("OUT_DIR").ok_or_else(|| invalid("Cargo did not supply OUT_DIR"))?,
    );
    let source = output.join(SOURCE_ROOT);
    if source.try_exists()? {
        ensure(
            !source.symlink_metadata()?.file_type().is_symlink(),
            "source output must not be a symbolic link",
        )?;
        fs::remove_dir_all(&source)?;
    }
    fs::create_dir(&source)?;
    unpack(&output)?;
    for addition in ADDITIONS {
        let relative = normal_path(Path::new(addition.target))?;
        ensure(
            relative.starts_with("src") && relative.extension().is_some_and(|e| e == "rs"),
            "added target must be an SDK Rust source file",
        )?;
        let path = source.join(relative);
        fs::create_dir_all(
            path.parent()
                .ok_or_else(|| invalid("added source has no parent"))?,
        )?;
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)?;
        file.write_all(addition.contents.as_bytes())?;
    }
    for patch in PATCHES {
        let relative = normal_path(Path::new(patch.target))?;
        ensure(
            relative.starts_with("src") && relative.extension().is_some_and(|e| e == "rs"),
            "patch target must be an SDK Rust source file",
        )?;
        let path = source.join(relative);
        ensure(
            !patch.before.is_empty(),
            "patch before-text must be nonempty",
        )?;
        let current = fs::read_to_string(&path)?;
        ensure(
            current.match_indices(patch.before).count() == 1,
            "patch before-text must occur exactly once",
        )?;
        fs::write(path, current.replacen(patch.before, patch.after, 1))?;
    }
    let root = fs::read_to_string(source.join("src/lib.rs"))?;
    let mut root = root
        .strip_prefix(ROOT_HEADER)
        .ok_or_else(|| invalid("upstream crate root attributes changed"))?
        .to_owned();
    let mut modules = vec![
        ("mod error;", "error.rs"),
        ("pub mod model;", "model.rs"),
        ("pub mod service;", "service.rs"),
        ("pub mod handler;", "handler.rs"),
        ("pub mod task_manager;", "task_manager.rs"),
        ("pub mod transport;", "transport.rs"),
    ];
    if root.contains("mod literal_json;") {
        modules.push(("mod literal_json;", "literal_json.rs"));
    }
    for (declaration, filename) in modules {
        ensure(
            root.match_indices(declaration).count() == 1,
            "upstream root module declaration changed",
        )?;
        // A #[path] pointing directly to model.rs makes its children resolve
        // beside that file. A mod.rs copy preserves the original model/ child
        // directory without rewriting any upstream nested module declarations.
        let original = source.join("src").join(filename);
        let module_name = Path::new(filename)
            .file_stem()
            .ok_or_else(|| invalid("root module has no stem"))?;
        let directory = source.join("src").join(module_name);
        fs::create_dir_all(&directory)?;
        let path = directory.join("mod.rs");
        ensure(
            !path.try_exists()?,
            "generated root module path already exists",
        )?;
        fs::copy(original, &path)?;
        let path = path
            .to_str()
            .ok_or_else(|| invalid("SDK build path must be Unicode"))?;
        root = root.replacen(
            declaration,
            &format!("#[path = {path:?}]\n{declaration}"),
            1,
        );
    }
    fs::write(output.join("rmcp-lib.rs"), root)?;
    Ok(())
}

fn unpack(output: &Path) -> io::Result<()> {
    const MAX_TAR_BYTES: u64 = 8 * 1024 * 1024;
    const MAX_FILE_BYTES: u64 = 1024 * 1024;
    let mut tar_bytes = Vec::new();
    GzDecoder::new(ARCHIVE)
        .take(MAX_TAR_BYTES + 1)
        .read_to_end(&mut tar_bytes)?;
    ensure(
        tar_bytes.len() as u64 <= MAX_TAR_BYTES,
        "decompressed SDK archive exceeds its byte bound",
    )?;
    let mut archive = tar::Archive::new(tar_bytes.as_slice());
    let mut paths = BTreeSet::new();
    let mut total = 0u64;
    for entry in archive.entries()? {
        let mut entry = entry?;
        ensure(
            entry.header().entry_type().is_file(),
            "SDK archive may contain regular files only",
        )?;
        let relative = normal_path(&entry.path()?)?;
        ensure(
            relative.starts_with(SOURCE_ROOT) && relative.components().count() >= 2,
            "SDK archive path has an unexpected root",
        )?;
        ensure(
            paths.insert(relative.clone()) && paths.len() <= 256,
            "duplicate SDK archive path or excessive file count",
        )?;
        let length = entry.size();
        total = total
            .checked_add(length)
            .ok_or_else(|| invalid("SDK archive byte count overflow"))?;
        ensure(
            length <= MAX_FILE_BYTES && total <= 4 * 1024 * 1024,
            "SDK archive file or total byte bound exceeded",
        )?;
        let destination = output.join(relative);
        fs::create_dir_all(
            destination
                .parent()
                .ok_or_else(|| invalid("SDK file has no parent"))?,
        )?;
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(destination)?;
        let written = io::copy(&mut entry, &mut file)?;
        ensure(written == length, "SDK archive file length differs")?;
        file.flush()?;
    }
    ensure(
        paths.len() == 181 && total == 3_504_281,
        "SDK archive entry count or uncompressed size differs",
    )
}

fn normal_path(path: &Path) -> io::Result<PathBuf> {
    ensure(
        !path.as_os_str().is_empty()
            && path.components().all(|c| matches!(c, Component::Normal(_))),
        "SDK paths must contain only normal relative components",
    )?;
    Ok(path.to_owned())
}
fn ensure(condition: bool, message: &str) -> io::Result<()> {
    if condition {
        Ok(())
    } else {
        Err(invalid(message))
    }
}
fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}
