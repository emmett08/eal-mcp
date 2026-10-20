use eal_core::{ExecutionLimits, composition, methods, semantic::Program};
use eal_runtime::{
    acquisition::{MAX_REQUEST_BYTES, ToolBinding, ToolRegistry},
    collection::{EvidenceRuntime, MAX_COLLECTION_CONTEXT_BYTES, acquisition_request},
    process::Supervisor,
    scheduler::CollectionScheduler,
    store::RunStore,
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};
use uuid::Uuid;

struct Directory(PathBuf);
impl Directory {
    fn new() -> Self {
        let p = std::env::temp_dir().join(format!("eal-collection-{}", Uuid::new_v4()));
        std::fs::create_dir(&p).unwrap();
        Self(p)
    }
}
impl Drop for Directory {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
fn source(count: usize, padding: usize) -> String {
    let mut s="language \"EAL/3\"\nenvironment lab {\nrequire site == \"bench\"\n}\ntool runner {\nversion \"1\"\n}\n".to_owned();
    for i in 0..count {
        let input = if i + 1 == count && padding != 0 {
            format!("input {{\"payload\":\"{}\"}}\n", "x".repeat(padding))
        } else {
            String::new()
        };
        s.push_str(&format!("evidence reading_{i} {{\ntool runner\nkind test\nenvironment lab\nmax_age 60\n{input}require passed == true\n}}\n"));
    }
    s.push_str("reasoning measured {\nmethod \"structured/1\"\nrationale \"The supplied observation supports this bounded claim.\"\n}\nclaim works {\nstatement \"The configured test passes.\"\nenvironment lab\n}\nargument support = [evidence reading_0] via measured => works\n");
    s
}
fn program(count: usize, padding: usize) -> Program {
    composition::compile(
        &source(count, padding),
        None,
        "",
        ExecutionLimits::default(),
    )
    .unwrap()
}
fn runtime(dir: &Path, binding: Option<ToolBinding>) -> EvidenceRuntime {
    let registry = ToolRegistry::new(
        binding
            .map(|b| BTreeMap::from([(b.name.clone(), b)]))
            .unwrap_or_default(),
    )
    .unwrap();
    EvidenceRuntime::new(
        dir.to_owned(),
        registry,
        RunStore::new(dir.join("runs.sqlite3")).unwrap(),
        methods::default_registry().clone(),
        CollectionScheduler::new(2).unwrap(),
        Supervisor::new(dir.join("unused-supervisor")),
    )
    .unwrap()
}
fn envelope(p: &Program, name: &str, context: &Value, value: Value) -> Value {
    json!({"observed_at":"2020-01-02T03:04:05+00:00","context":context,"request":acquisition_request(p,name,context).unwrap(),"value":value})
}
fn no_records(runtime: &EvidenceRuntime) {
    assert!(
        runtime
            .store()
            .list(Some("observation"), 1000)
            .unwrap()
            .is_empty()
    );
    assert!(
        runtime
            .store()
            .list(Some("collection"), 1000)
            .unwrap()
            .is_empty()
    );
}

#[test]
fn imported_measurement_keeps_original_time_and_only_stream_hashes_are_durable() {
    let dir = Directory::new();
    let p = program(1, 0);
    let context = json!({"site":"bench"});
    let input = envelope(&p, "reading_0", &context, json!({"passed":true}));
    let bytes = serde_json::to_vec(&input).unwrap();
    std::fs::write(dir.0.join("value.json"), &bytes).unwrap();
    let runtime = runtime(
        &dir.0,
        Some(ToolBinding::json_file("runner", "1", "value.json")),
    );
    let collected = runtime.collect(&p, &context, None).unwrap();
    let record = &collected["records"]["reading_0"];
    assert_eq!(record["status"], "ok");
    assert_eq!(record["collected_at"], input["observed_at"]);
    assert_ne!(record["ingested_at"], record["collected_at"]);
    assert_ne!(record["started_at"], record["collected_at"]);
    assert!(record.get("stdout").is_none());
    assert!(record.get("stderr").is_none());
    assert_eq!(record["stdout_bytes"], bytes.len());
    assert_eq!(record["stderr_bytes"], 0);
    assert_eq!(
        record["stdout_digest"],
        format!("{:x}", Sha256::digest(&bytes))
    );
    assert_eq!(
        runtime
            .store()
            .get(record["run_id"].as_str().unwrap(), Some("observation"))
            .unwrap(),
        *record
    );
    let stored = runtime
        .store()
        .get(
            collected["collection_id"].as_str().unwrap(),
            Some("collection"),
        )
        .unwrap();
    assert_eq!(stored["records"]["reading_0"], *record);
    assert!(stored.get("collection_id").is_none());
}

#[test]
fn failed_and_missing_bindings_are_durable_error_observations() {
    for missing in [false, true] {
        let dir = Directory::new();
        let p = program(1, 0);
        std::fs::write(dir.0.join("value.json"), b"private non-JSON stream").unwrap();
        let runtime = runtime(
            &dir.0,
            (!missing).then(|| ToolBinding::json_file("runner", "1", "value.json")),
        );
        let collected = runtime.collect(&p, &json!({"site":"bench"}), None).unwrap();
        let record = &collected["records"]["reading_0"];
        assert_eq!(record["status"], "error");
        assert!(record["error"]["message"].is_string());
        assert!(record.get("value").is_none());
        assert!(record.get("stdout").is_none());
        assert!(record.get("stderr").is_none());
        assert_eq!(
            runtime
                .store()
                .get(record["run_id"].as_str().unwrap(), Some("observation"))
                .unwrap(),
            *record
        );
        assert_eq!(
            runtime
                .store()
                .list(Some("observation"), 100)
                .unwrap()
                .len(),
            1
        );
        assert_eq!(
            runtime.store().list(Some("collection"), 100).unwrap().len(),
            1
        );
    }
}

#[test]
fn validation_context_request_and_aggregate_allowances_fail_before_any_observation() {
    let dir = Directory::new();
    let mut binding = ToolBinding::json_file("runner", "1", "missing.json");
    binding.max_output_bytes = 16 * 1024 * 1024;
    let runtime = runtime(&dir.0, Some(binding));
    let p = program(1, 0);
    assert!(
        runtime
            .collect(
                &p,
                &json!({"site":"bench","padding":"x".repeat(MAX_COLLECTION_CONTEXT_BYTES)}),
                None
            )
            .unwrap_err()
            .contains("Collection context exceeds")
    );
    no_records(&runtime);
    let p = program(129, 0);
    assert!(
        runtime
            .collect(&p, &json!({"site":"bench"}), None)
            .unwrap_err()
            .contains("128 evidence requests")
    );
    no_records(&runtime);
    let p = program(2, MAX_REQUEST_BYTES - 2000);
    let error = runtime
        .collect(
            &p,
            &json!({"site":"bench","padding":"y".repeat(3000)}),
            None,
        )
        .unwrap_err();
    assert!(
        error.contains("Tool request for 'reading_1' exceeds"),
        "{error}"
    );
    no_records(&runtime);
    let p = program(9, 0);
    assert!(
        runtime
            .collect(&p, &json!({"site":"bench"}), None)
            .unwrap_err()
            .contains("Collection output allowance exceeds")
    );
    no_records(&runtime);
    let invalid = composition::compile(
        &source(1, 0).replace("structured/1", "unknown/1"),
        None,
        "",
        ExecutionLimits::default(),
    )
    .unwrap();
    assert!(
        runtime
            .collect(&invalid, &json!({"site":"bench"}), None)
            .unwrap_err()
            .contains("invalid program")
    );
    no_records(&runtime);
}

#[test]
fn selected_requests_are_unique_declared_and_returned_in_requested_order() {
    let dir = Directory::new();
    let p = program(2, 0);
    let runtime = runtime(&dir.0, None);
    let context = json!({"site":"bench"});
    for selected in [
        vec!["reading_0".into(), "reading_0".into()],
        vec!["absent".into()],
    ] {
        assert!(
            runtime
                .collect(&p, &context, Some(&selected))
                .unwrap_err()
                .contains("unique declared")
        );
        no_records(&runtime);
    }
    let selected = vec!["reading_1".into(), "reading_0".into()];
    let result = runtime.collect(&p, &context, Some(&selected)).unwrap();
    assert_eq!(
        result["records"]
            .as_object()
            .unwrap()
            .keys()
            .cloned()
            .collect::<Vec<_>>(),
        selected
    );
}

#[test]
fn final_collection_byte_failure_retains_all_individually_durable_observations() {
    let dir = Directory::new();
    let mut p = program(3, 0);
    p.limits.record_bytes = 16 * 1024 * 1024;
    let context = json!({"site":"bench"});
    let input = envelope(
        &p,
        "reading_0",
        &context,
        json!({"passed":true,"padding":"x".repeat(11*1024*1024)}),
    );
    std::fs::write(
        dir.0.join("value.json"),
        serde_json::to_vec(&input).unwrap(),
    )
    .unwrap();
    let mut binding = ToolBinding::json_file("runner", "1", "value.json");
    binding.max_output_bytes = 16 * 1024 * 1024;
    let runtime = runtime(&dir.0, Some(binding));
    let error = runtime.collect(&p, &context, None).unwrap_err();
    assert!(
        error.contains("Collection exceeds 33554432 stored bytes"),
        "{error}"
    );
    assert_eq!(
        runtime
            .store()
            .list(Some("observation"), 100)
            .unwrap()
            .len(),
        3
    );
    assert!(
        runtime
            .store()
            .list(Some("collection"), 100)
            .unwrap()
            .is_empty()
    );
}
