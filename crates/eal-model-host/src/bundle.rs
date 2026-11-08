//! Immutable local artifact identities checked before model deserialisation.
use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs::{self, File},
    io::Read,
    path::{Component, Path},
};

const PINNED_MODEL: &str = include_str!("../models/pinned-model.json");

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Artifact {
    sha256: String,
    bytes: u64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BaseModel {
    model_id: String,
    revision: String,
    files: BTreeMap<String, Artifact>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Adapter {
    name: String,
    files: BTreeMap<String, Artifact>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PinnedModel {
    schema: String,
    base_model: BaseModel,
    adapter: Adapter,
}

/// Hash local bytes without deserialising tensors, checkpoints or executable code.
pub fn sha256(path: &Path) -> Result<String> {
    let mut file = File::open(path).with_context(|| format!("open {}", path.display()))?;
    let mut digest = Sha256::new();
    let mut buffer = vec![0u8; 256 * 1024];
    loop {
        let count = file.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        digest.update(&buffer[..count]);
    }
    Ok(format!("{:x}", digest.finalize()))
}

fn verify_files(root: &Path, artifacts: &BTreeMap<String, Artifact>) -> Result<Value> {
    ensure!(
        root.is_dir(),
        "artifact root must be a directory: {}",
        root.display()
    );
    ensure!(!artifacts.is_empty(), "artifact manifest is empty");
    let mut verified = serde_json::Map::new();
    for (relative, expected) in artifacts {
        let path = Path::new(relative);
        ensure!(
            !relative.is_empty()
                && path
                    .components()
                    .all(|part| matches!(part, Component::Normal(_))),
            "artifact path must be relative normal components: {relative}"
        );
        ensure!(
            expected.sha256.len() == 64 && expected.sha256.bytes().all(|c| c.is_ascii_hexdigit()),
            "invalid SHA256 pin for {relative}"
        );
        let path = root.join(path);
        let metadata =
            fs::metadata(&path).with_context(|| format!("required artifact {relative}"))?;
        ensure!(
            metadata.is_file(),
            "artifact must be a regular file: {relative}"
        );
        ensure!(
            metadata.len() == expected.bytes,
            "artifact size mismatch: {relative}"
        );
        let actual = sha256(&path)?;
        ensure!(
            actual == expected.sha256,
            "artifact SHA256 mismatch: {relative}"
        );
        verified.insert(
            relative.clone(),
            json!({"sha256":actual,"bytes":metadata.len()}),
        );
    }
    Ok(Value::Object(verified))
}

/// Verify the supported original adapter and base against compiled-in pins.
///
/// Run directories and Hugging Face cache symlinks are operator-owned local files.
/// No network requests or Python checkpoint loading occur here. Keep these files
/// immutable between verification and inference, including symlink targets.
pub fn verify_pinned_bundle(base_dir: &Path, adapter_dir: &Path) -> Result<Value> {
    let pins: PinnedModel = serde_json::from_str(PINNED_MODEL)?;
    ensure!(
        pins.schema == "eal.pinned-local-model/1",
        "unsupported pin schema"
    );
    let base_files = verify_files(base_dir, &pins.base_model.files).context("pinned base model")?;
    let adapter_files =
        verify_files(adapter_dir, &pins.adapter.files).context("pinned LoRA adapter")?;
    Ok(json!({
        "schema":"eal.local-model-verification/1",
        "status":"verified",
        "manifest_sha256":format!("{:x}",Sha256::digest(PINNED_MODEL.as_bytes())),
        "base_model":{"model_id":pins.base_model.model_id,"revision":pins.base_model.revision,
            "files":base_files},
        "adapter":{"name":pins.adapter.name,"files":adapter_files},
        "scope":"local artifact identity; no model-quality or task-success claim"
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn one_artifact() -> BTreeMap<String, Artifact> {
        BTreeMap::from([(
            "asset.bin".into(),
            Artifact {
                // Independently specified SHA256 test vector for the three ASCII bytes abc.
                sha256: "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad".into(),
                bytes: 3,
            },
        )])
    }

    #[test]
    fn verifies_bytes_and_reports_the_identity() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("asset.bin"), b"abc").unwrap();
        let result = verify_files(dir.path(), &one_artifact()).unwrap();
        assert_eq!(result["asset.bin"]["bytes"], 3);
        assert_eq!(
            result["asset.bin"]["sha256"],
            one_artifact()["asset.bin"].sha256
        );
    }

    #[test]
    fn rejects_missing_truncated_and_same_size_modified_files() {
        let dir = tempfile::tempdir().unwrap();
        assert!(verify_files(dir.path(), &one_artifact()).is_err());
        fs::write(dir.path().join("asset.bin"), b"ab").unwrap();
        assert!(
            verify_files(dir.path(), &one_artifact())
                .unwrap_err()
                .to_string()
                .contains("size mismatch")
        );
        fs::write(dir.path().join("asset.bin"), b"abd").unwrap();
        assert!(
            verify_files(dir.path(), &one_artifact())
                .unwrap_err()
                .to_string()
                .contains("SHA256 mismatch")
        );
    }

    #[test]
    fn rejects_directories_and_escape_components() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir(dir.path().join("asset.bin")).unwrap();
        assert!(verify_files(dir.path(), &one_artifact()).is_err());
        for path in ["../asset.bin", "/asset.bin", "", "./asset.bin"] {
            let mut entries = one_artifact();
            let artifact = entries.remove("asset.bin").unwrap();
            entries.insert(path.into(), artifact);
            assert!(verify_files(dir.path(), &entries).is_err());
        }
    }

    #[test]
    fn pinned_manifest_covers_every_runtime_input() {
        let pins: PinnedModel = serde_json::from_str(PINNED_MODEL).unwrap();
        assert_eq!(pins.base_model.files.len(), 3);
        assert_eq!(pins.adapter.files.len(), 12);
        for path in ["model.safetensors", "config.json", "generation_config.json"] {
            assert!(pins.base_model.files.contains_key(path));
        }
        for path in [
            "adapter/adapter_model.safetensors",
            "adapter/adapter_config.json",
            "tokenizer/tokenizer.json",
            "tokenizer/tokenizer_config.json",
            "tokenizer/chat_template.jinja",
            "run_identity.json",
            "training_report.json",
        ] {
            assert!(pins.adapter.files.contains_key(path));
        }
        assert_eq!(
            pins.adapter.files["adapter/adapter_model.safetensors"].sha256,
            "2630180e0ede1da7036a14a217348a505fcf2b5b8d27d8b89a39db68a5cd2bcf"
        );
    }
}
