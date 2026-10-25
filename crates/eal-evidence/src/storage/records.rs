//! Immutable signed observations, followed by atomic publication of the latest index.
use super::*;
impl SourceStore {
    pub(crate) fn load(&self, key: &str) -> Result<Option<SourceRecord>, String> {
        if !valid_digest(key) {
            return Err("source identity is invalid".into());
        }
        let Some(mut record) = self.read_record(&self.root.join(format!("{key}.json")))? else {
            return Ok(None);
        };
        if record.source_id != key {
            return Err("stored source identity differs from its reuse index".into());
        }
        // Admit older signed cache entries by retaining their original data as
        // an immutable observation before they can be referenced again.
        if record.observation_id.is_empty() {
            self.seal(&mut record)?;
            record = self.save(&record)?;
        }
        let snapshot = self
            .load_observation(&record.observation_id)?
            .ok_or("stored source observation snapshot is absent")?;
        if snapshot.observation_identity()? != record.observation_identity()? {
            return Err("source index differs from its immutable observation".into());
        }
        Ok(Some(record))
    }
    /// Load an exact retained observation independently of the latest source
    /// index, verifying its identity, complete record signature and original age.
    pub fn load_observation(&self, id: &str) -> Result<Option<SourceRecord>, String> {
        if !valid_digest(id) {
            return Err("observation identity is invalid".into());
        }
        private_directory(&self.root.join("observations"))?;
        let record =
            self.read_record(&self.root.join("observations").join(format!("{id}.json")))?;
        if record
            .as_ref()
            .is_some_and(|record| record.observation_id != id)
        {
            return Err("stored observation identity differs from its immutable path".into());
        }
        Ok(record)
    }
    fn read_record(&self, path: &Path) -> Result<Option<SourceRecord>, String> {
        match fs::symlink_metadata(path) {
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(_) => return Err("source record could not be inspected".into()),
        }
        let file = open_private(path, false)?;
        if file
            .metadata()
            .map_err(|_| "source record metadata is unavailable")?
            .len()
            > self.maximum_bytes as u64
        {
            return Err("source record exceeds its byte limit".into());
        }
        let mut bytes = Vec::new();
        file.take(self.maximum_bytes as u64 + 1)
            .read_to_end(&mut bytes)
            .map_err(|_| "source record could not be read")?;
        if bytes.len() > self.maximum_bytes {
            return Err("source record exceeds its byte limit".into());
        }
        let value = eal_collector_contracts::strict_json_exact(
            &bytes,
            eal_collector_contracts::JsonLimits {
                max_bytes: self.maximum_bytes,
                ..Default::default()
            },
        )?;
        let record: SourceRecord =
            serde_json::from_value(value).map_err(|_| "source record has invalid fields")?;
        self.verify(&record)?;
        Ok(Some(record))
    }
    fn verify(&self, record: &SourceRecord) -> Result<(), String> {
        if record.schema != "eal.shared-source/1"
            || !valid_digest(&record.source_id)
            || canonical_digest(&(&record.source, &record.arguments))? != record.source_id
            || canonical_digest(&record.value)? != record.value_sha256
            || (!record.observation_id.is_empty()
                && record.observation_identity()? != record.observation_id)
        {
            return Err(
                "stored source/observation integrity differs from its retained identity or value digest"
                    .into(),
            );
        }
        if !valid_digest(&record.integrity) {
            return Err("stored source integrity is missing".into());
        }
        let signature = decode_hex(&record.integrity)?;
        let mut verifier = Hmac::<Sha256>::new_from_slice(self.key.as_slice())
            .map_err(|_| "source store integrity key is invalid")?;
        verifier.update(self.seal_bytes(record)?.as_bytes());
        verifier
            .verify_slice(&signature)
            .map_err(|_| "stored source record integrity failed")?;
        record.source.validate()?;
        chrono::DateTime::parse_from_rfc3339(&record.observed_at)
            .map_err(|_| "stored original measurement time is invalid")?;
        Ok(())
    }
    pub(crate) fn save(&self, record: &SourceRecord) -> Result<SourceRecord, String> {
        self.verify(record)?;
        if !valid_digest(&record.observation_id) {
            return Err("source record requires an immutable observation identity".into());
        }
        let bytes = serde_json::to_vec(record).map_err(|_| "source record could not be encoded")?;
        if bytes.len() > self.maximum_bytes {
            return Err("source record exceeds its byte limit".into());
        }
        let _store_lock = open_private(&self.root.join("store.lock"), true)?;
        #[cfg(unix)]
        rustix::fs::flock(&_store_lock, rustix::fs::FlockOperation::LockExclusive)
            .map_err(|_| "source store publication lock failed")?;
        let directory = self.root.join("observations");
        private_directory(&directory)?;
        let destination = self.root.join(format!("{}.json", record.source_id));
        if !destination.exists()
            && count_json(&self.root, self.maximum_records)? >= self.maximum_records
        {
            return Err("source store reuse-index capacity is exhausted; existing observations were retained".into());
        }
        let snapshot = directory.join(format!("{}.json", record.observation_id));
        let canonical = if let Some(existing) = self.load_observation(&record.observation_id)? {
            existing
        } else {
            if count_json(&directory, self.maximum_records)? >= self.maximum_records {
                return Err("source store observation capacity is exhausted; existing observations were retained".into());
            }
            publish(&directory, &snapshot, &bytes)?;
            record.clone()
        };
        // Fetch costs belong to execution metrics. Repeated reads of the exact
        // same measurement retain the first signed snapshot byte-for-byte.
        let bytes =
            serde_json::to_vec(&canonical).map_err(|_| "source record could not be encoded")?;
        // A snapshot is durable before a reuse index can expose its ID. No
        // snapshot is overwritten or evicted to make room for a fresh read.
        publish(&self.root, &destination, &bytes)?;
        Ok(canonical)
    }
}
fn count_json(path: &Path, maximum: usize) -> Result<usize, String> {
    let mut count = 0;
    for entry in fs::read_dir(path).map_err(|_| "source store directory is unavailable")? {
        let entry = entry.map_err(|_| "source store entry is unavailable")?;
        if entry
            .path()
            .extension()
            .is_some_and(|extension| extension == "json")
        {
            count += 1;
            if count >= maximum {
                break;
            }
        }
    }
    Ok(count)
}
fn publish(directory: &Path, destination: &Path, bytes: &[u8]) -> Result<(), String> {
    let temporary = directory.join(format!(
        ".source-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let result = (|| {
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let mut file = options
            .open(&temporary)
            .map_err(|_| "source record could not be reserved")?;
        file.write_all(bytes)
            .map_err(|_| "source record could not be written")?;
        file.sync_all()
            .map_err(|_| "source record could not be synced")?;
        fs::rename(&temporary, destination).map_err(|_| "source record could not be published")?;
        File::open(directory)
            .and_then(|file| file.sync_all())
            .map_err(|_| "source store directory could not be synced")?;
        Ok(())
    })();
    let _ = fs::remove_file(&temporary);
    result
}
