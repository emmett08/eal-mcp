use super::{RegisteredAssessmentHost, Selection};
use crate::{collection::MAX_COLLECTION_BYTES, store::StoreEntry};
use serde_json::{Value, json};
use std::collections::BTreeMap;

impl RegisteredAssessmentHost {
    pub(super) fn persist_mixed(
        &self,
        selected: &Selection,
        reused: &BTreeMap<String, Value>,
        fresh: &BTreeMap<String, Value>,
    ) -> Result<String, String> {
        let mut records = serde_json::Map::new();
        for name in &selected.names {
            records.insert(
                name.clone(),
                reused
                    .get(name)
                    .or_else(|| fresh.get(name))
                    .ok_or("Planned acquisition is missing from the final collection")?
                    .clone(),
            );
        }
        let collection = json!({"source_digest":selected.program.source_digest,"context":selected.context,"records":records});
        if crate::collection::stored_collection_size(&collection, &selected.program.limits)?
            > MAX_COLLECTION_BYTES
        {
            return Err(format!(
                "Collection exceeds {MAX_COLLECTION_BYTES} stored bytes"
            ));
        }
        if reused.is_empty() {
            return self.service.store().put("collection", &collection, None);
        }
        let mut entries = Vec::new();
        for name in &selected.names {
            if let Some(record) = reused.get(name) {
                entries.push(StoreEntry {
                    kind: "observation",
                    payload: record,
                    record_id: Some(
                        record["run_id"]
                            .as_str()
                            .ok_or("Rebound observation has no run_id")?,
                    ),
                });
            }
        }
        entries.push(StoreEntry {
            kind: "collection",
            payload: &collection,
            record_id: None,
        });
        self.service
            .store()
            .put_batch(&entries)?
            .pop()
            .ok_or("Mixed collection was not persisted".into())
    }
}
