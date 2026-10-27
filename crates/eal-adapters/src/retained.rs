//! Pinned artifact imports retain original measurement identities and clocks.
mod review;
use crate::Error;
pub use review::{REVIEW_INPUT_SCHEMA, Review, ReviewInputs, import_review};
use serde::Deserialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    path::{Component, Path, PathBuf},
};
const MAX_FILE_BYTES: usize = 1_048_576;
const MAX_ARTIFACT_BYTES: usize = 4_194_304;
const MAX_ARTIFACTS: usize = 16;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Artifact {
    pub path: PathBuf,
    pub sha256: String,
    pub observed_at: String,
    pub source: String,
}

fn invalid(message: &str) -> Error {
    Error::InvalidInput(message.into())
}

fn text(value: &str, max: usize) -> bool {
    !value.trim().is_empty() && value.len() <= max && !value.chars().any(char::is_control)
}

fn digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

fn hash(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn artifact_path(base: &Path, path: &Path) -> Result<PathBuf, Error> {
    if path.to_str().is_none_or(|value| !text(value, 4096)) {
        return Err(invalid("retained artifact path is missing or invalid"));
    }
    if path
        .components()
        .any(|part| matches!(part, Component::ParentDir))
    {
        return Err(invalid(
            "retained artifact paths must not traverse parent directories",
        ));
    }
    let selected = if path.is_absolute() {
        path.to_owned()
    } else {
        base.join(path)
    };
    if std::fs::symlink_metadata(&selected)?
        .file_type()
        .is_symlink()
    {
        return Err(invalid("retained artifact must not be a symbolic link"));
    }
    let canonical = selected.canonicalize()?;
    if !path.is_absolute() && !canonical.starts_with(base) {
        return Err(invalid(
            "relative retained artifact escapes the inputs directory",
        ));
    }
    Ok(canonical)
}

/// A sortable retained UTC timestamp, with nanoseconds padded for comparison.
pub fn timestamp(value: &str) -> Result<(String, String), Error> {
    let bytes = value.as_bytes();
    let expected = [(4, b'-'), (7, b'-'), (10, b'T'), (13, b':'), (16, b':')];
    if !(20..=30).contains(&bytes.len())
        || expected.iter().any(|(i, b)| bytes.get(*i) != Some(b))
        || bytes.last() != Some(&b'Z')
        || bytes[..19].iter().enumerate().any(|(i, b)| {
            !expected.iter().any(|(separator, _)| *separator == i) && !b.is_ascii_digit()
        })
    {
        return Err(invalid(
            "observed_at requires a retained UTC RFC3339 timestamp",
        ));
    }
    let number = |start: usize, end: usize| -> u32 {
        bytes[start..end]
            .iter()
            .fold(0, |n, b| n * 10 + u32::from(b - b'0'))
    };
    let (year, month, day) = (number(0, 4), number(5, 7), number(8, 10));
    let leap = year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400));
    let days = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        2 => 28,
        _ => 0,
    };
    if year == 0
        || day == 0
        || day > days
        || number(11, 13) > 23
        || number(14, 16) > 59
        || number(17, 19) > 59
    {
        return Err(invalid("observed_at contains an invalid calendar instant"));
    }
    let fraction = if bytes.len() == 20 {
        ""
    } else {
        if bytes[19] != b'.'
            || bytes.len() < 22
            || !bytes[20..bytes.len() - 1].iter().all(u8::is_ascii_digit)
        {
            return Err(invalid("observed_at contains an invalid fractional second"));
        }
        &value[20..value.len() - 1]
    };
    Ok((value[..19].into(), format!("{fraction:0<9}")))
}

/// Resolve relative artifact paths beneath one canonical operator-selected base.
/// Absolute artifact pins are independent grants and do not consult `base`.
pub fn import_artifacts(artifacts: &[Artifact], base: &Path) -> Result<(Value, String), Error> {
    if artifacts.is_empty() || artifacts.len() > MAX_ARTIFACTS {
        return Err(invalid(
            "each evidence need requires 1 to 16 retained artifacts",
        ));
    }
    let canonical_base = artifacts
        .iter()
        .any(|artifact| artifact.path.is_relative())
        .then(|| base.canonicalize())
        .transpose()?;
    let base = canonical_base.as_deref().unwrap_or(base);
    if canonical_base.is_some() && !base.is_dir() {
        return Err(invalid("retained artifact base requires a directory"));
    }
    let mut values = Vec::new();
    let mut paths = BTreeSet::new();
    let mut total = 0usize;
    let mut oldest = None;
    for artifact in artifacts {
        if !digest(&artifact.sha256) || !text(&artifact.source, 4096) {
            return Err(invalid(
                "retained artifacts require SHA-256 pins and source provenance",
            ));
        }
        let instant = timestamp(&artifact.observed_at)?;
        let path = artifact_path(base, &artifact.path)?;
        if !paths.insert(path.clone()) {
            return Err(invalid("a retained artifact path is listed more than once"));
        }
        let bytes = crate::files::read(&path, MAX_FILE_BYTES, "retained artifact")?;
        total = total.saturating_add(bytes.len());
        if total > MAX_ARTIFACT_BYTES {
            return Err(Error::Budget(
                "retained artifacts exceed 4 MiB in aggregate".into(),
            ));
        }
        if hash(&bytes) != artifact.sha256 {
            return Err(invalid(
                "retained artifact bytes differ from their SHA-256 pin",
            ));
        }
        let observation = crate::parse_json(&bytes)?;
        if !observation.is_object() {
            return Err(invalid(
                "retained artifact observation must be a JSON object",
            ));
        }
        if oldest.as_ref().is_none_or(|(old, _)| instant < *old) {
            oldest = Some((instant, artifact.observed_at.clone()));
        }
        values.push(json!({"path":artifact.path,"sha256":artifact.sha256,
            "observed_at":artifact.observed_at,"source":artifact.source,"observation":observation}));
    }
    Ok((
        json!({"collected":true,"artifacts":values,
        "interpretation":"retained_artifacts_require_independent_review"}),
        oldest.expect("nonempty artifacts validated").1,
    ))
}
