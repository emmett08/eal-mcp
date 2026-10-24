//! Explicit revision admission and native executable identity.
use super::ensure;
use crate::process::{executable, execute};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};
pub(super) fn revision(root: &Path, expected: &str, git: &Path) -> Result<(), String> {
    let result = execute(
        git,
        &["rev-parse".into(), "HEAD".into()],
        None,
        Some(root),
        false,
        0,
    )?;
    let actual = String::from_utf8(result.stdout).map_err(|e| e.to_string())?;
    ensure(
        actual.trim() == expected,
        &format!("Expected revision {expected}, found {}", actual.trim()),
    )?;
    let result = execute(
        git,
        &[
            "status".into(),
            "--porcelain".into(),
            "--untracked-files=no".into(),
        ],
        None,
        Some(root),
        false,
        0,
    )?;
    ensure(
        result.stdout.is_empty(),
        "Pinned checkout has modified tracked files",
    )
}
pub(super) fn binary_hash(path: &Path) -> Result<String, String> {
    use sha2::{Digest, Sha256};
    use std::io::Read;
    let mut file =
        fs::File::open(path).map_err(|e| format!("Cannot read {}: {e}", path.display()))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 65536];
    loop {
        let count = file.read(&mut buffer).map_err(|e| e.to_string())?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}
pub(super) struct Options {
    pub(super) candidate: PathBuf,
    pub(super) reference: PathBuf,
    pub(super) candidate_revision: String,
    pub(super) reference_revision: String,
    pub(super) output: PathBuf,
    pub(super) git: PathBuf,
    pub(super) cargo: Option<PathBuf>,
}
pub(super) fn options(args: &[String]) -> Result<Options, String> {
    let mut values = BTreeMap::new();
    let mut build = false;
    let mut i = 0;
    while i < args.len() {
        if args[i] == "--build" {
            if build {
                return Err("Duplicate --build".into());
            }
            build = true;
            i += 1;
            continue;
        }
        let flag = &args[i];
        if !matches!(
            flag.as_str(),
            "--candidate-root"
                | "--reference-root"
                | "--candidate-revision"
                | "--reference-revision"
                | "--output"
                | "--git"
                | "--cargo"
        ) {
            return Err(format!("Unknown probe option {flag}"));
        }
        let value = args
            .get(i + 1)
            .ok_or_else(|| format!("Missing value for {flag}"))?;
        if values.insert(flag.clone(), value.clone()).is_some() {
            return Err(format!("Duplicate option {flag}"));
        }
        i += 2;
    }
    let root = |flag: &str| {
        values
            .get(flag)
            .ok_or_else(|| format!("Required {flag}"))
            .and_then(|path| fs::canonicalize(path).map_err(|e| e.to_string()))
    };
    let candidate = root("--candidate-root")?;
    let reference = root("--reference-root")?;
    let revision = |flag: &str| {
        let value = values
            .get(flag)
            .ok_or_else(|| format!("Required {flag} SHA40"))?;
        if value.len() != 40 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err(format!(
                "{flag} must be an exact 40-digit hexadecimal revision"
            ));
        }
        Ok(value.to_ascii_lowercase())
    };
    let candidate_revision = revision("--candidate-revision")?;
    let reference_revision = revision("--reference-revision")?;
    ensure(
        candidate != reference,
        "Reference and candidate checkouts must be distinct",
    )?;
    let output = PathBuf::from(values.get("--output").ok_or("Required --output NEW_FILE")?);
    ensure(
        !output.exists(),
        "Output must be a new file; historical results are retained",
    )?;
    let git = executable(values.get("--git").map(String::as_str).unwrap_or("git"))?;
    let cargo = if build {
        Some(executable(
            values.get("--cargo").map(String::as_str).unwrap_or("cargo"),
        )?)
    } else {
        None
    };
    if !build && values.contains_key("--cargo") {
        return Err("--cargo requires --build".into());
    }
    Ok(Options {
        candidate,
        reference,
        candidate_revision,
        reference_revision,
        output,
        git,
        cargo,
    })
}
