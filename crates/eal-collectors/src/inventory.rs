//! One identified, finite operator declaration of the production cluster universe.
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Cluster {
    pub id: String,
    pub context: String,
    pub tempo_cluster: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Inventory {
    pub revision: String,
    pub clusters: Vec<Cluster>,
}
pub(crate) fn read(config: &crate::Config) -> Result<(Inventory, String), String> {
    let path = config.path("--inventory")?;
    // Resolve operator-selected parent aliases while leaving the leaf untouched
    // so the runtime reader can reject a symlink or special file before reading.
    let name = path
        .file_name()
        .ok_or("Inventory must name a regular file")?;
    let parent = path
        .parent()
        .ok_or("Inventory requires an absolute parent directory")?
        .canonicalize()
        .map_err(|_| "Cannot resolve cluster inventory parent")?;
    let path = parent.join(name);
    let bytes = eal_runtime::acquisition::read_regular(
        &path,
        1_048_576,
        "Inventory must be a bounded regular file",
    )
    .map_err(|_| "Cannot read the bounded regular no-follow cluster inventory")?;
    if bytes.len() > 1_048_576 {
        return Err("Cluster inventory exceeds 1 MiB".into());
    }
    let value = eal_runtime::acquisition::strict_json(
        std::str::from_utf8(&bytes).map_err(|_| "Inventory requires UTF-8")?,
    )
    .map_err(|_| "Inventory must be strict JSON")?;
    let inventory: Inventory =
        serde_json::from_value(value).map_err(|_| "Invalid cluster inventory contract")?;
    if inventory.revision.is_empty()
        || inventory.revision.len() > 1024
        || inventory.clusters.is_empty()
        || inventory.clusters.len() > 256
    {
        return Err("Inventory requires a revision and 1 to 256 clusters".into());
    }
    let mut ids = BTreeSet::new();
    let mut contexts = BTreeSet::new();
    let mut telemetry = BTreeSet::new();
    for cluster in &inventory.clusters {
        for value in [&cluster.id, &cluster.context, &cluster.tempo_cluster] {
            if value.is_empty()
                || value.len() > 253
                || value.chars().any(char::is_control)
                || value.starts_with('-')
            {
                return Err("Invalid cluster identity".into());
            }
        }
        if !ids.insert(&cluster.id)
            || !contexts.insert(&cluster.context)
            || !telemetry.insert(&cluster.tempo_cluster)
        {
            return Err("Inventory cluster identities must be unique".into());
        }
    }
    Ok((inventory, format!("{:x}", Sha256::digest(bytes))))
}
