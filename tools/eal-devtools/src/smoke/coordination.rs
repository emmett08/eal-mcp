//! Check installed coordination lifecycle, exact first responses and unknown-effect recovery.
use super::{Options, Result, ensure, harness::Harness, read_json, write_json};
use base64::{Engine, engine::general_purpose::STANDARD};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
};
struct Run<'a> {
    h: &'a Harness,
    worker: PathBuf,
    entry: Value,
    task: Value,
    server: Value,
}
impl Run<'_> {
    fn command(&self, op: &str, dir: &Path, args: &[&str]) -> Result<Value> {
        self.h.coordinate(
            op,
            dir,
            &args.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
            0,
            false,
        )
    }
    fn execute(&self, dir: &Path) -> Result<Value> {
        self.command(
            "execute",
            dir,
            &[
                "--execution-mode",
                "software_test",
                "--worker-argv",
                &json!([self.worker]).to_string(),
                "--worker-environment",
                &self
                    .h
                    .root
                    .join("worker-environment.json")
                    .display()
                    .to_string(),
            ],
        )
    }
    fn spec(&self, label: &str, transport: Value) -> Result<PathBuf> {
        let path = self.h.root.join(format!("{label}-spec.json"));
        write_json(
            &path,
            &json!({"server":self.server,"task":self.task,"transport":transport}),
        )?;
        Ok(path)
    }
    fn export(&self, dir: &Path, label: &str) -> Result<Value> {
        let path = self.h.root.join(format!("{label}-export.json"));
        let value = self.command("export", dir, &["--output", &path.display().to_string()])?;
        ensure(
            read_json(&path)? == value,
            "Export bytes differed from response",
        )?;
        Ok(value)
    }
    fn completed(&self, dir: &Path, label: &str, expected_collected: u64) -> Result<Value> {
        let status = self.execute(dir)?;
        ensure(
            status["status"] == "completed",
            "Coordination did not complete",
        )?;
        let value = self.export(dir, label)?;
        let rows = effects(&value)?;
        let finding = &rows["assessment"]["result"];
        ensure(
            finding["check"]["ok"] == true,
            "Finding correspondence check failed",
        )?;
        let assessment = &finding["assessments"]["works"];
        let expected = json!({"task_digest":value["config"]["anchor"]["task_digest"],"finding_id":finding["finding_id"],"claims":{"works":{"assessment_id":assessment["assessment_id"],"status":assessment["status"],"assessed_at":assessment["assessed_at"]}},"scope":{"context":{"site":"bench"},"requirements":self.task["requirements"]},"conclusion":"assessed","qualification":"conditional_on_registered_argument"});
        ensure(
            status["answer"] == expected,
            "Completed answer lost assessment identity or scope",
        )?;
        ensure(
            assessment["status"] == "supported"
                && assessment["source_digest"] == self.entry["source_digest"],
            "Assessment revision mismatch",
        )?;
        for component in [finding, assessment] {
            ensure(
                component["collected_count"] == expected_collected
                    && component["reused_count"] == 1 - expected_collected,
                "Unexpected collection/reuse accounting",
            )?;
        }
        ensure(
            assessment["packet"] == finding["packets"]["works"]
                && finding["packets"]["works"]["assessment_id"] == assessment["assessment_id"]
                && finding["packets"]["works"]["source_digest"] == assessment["source_digest"]
                && finding["explanations"]["works"]["assessed_at"] == assessment["assessed_at"],
            "Packet/explanation correspondence mismatch",
        )?;
        let assessed = chrono::DateTime::parse_from_rfc3339(
            assessment["assessed_at"]
                .as_str()
                .ok_or("Missing assessment time")?,
        )
        .map_err(|e| e.to_string())?;
        let pinned = chrono::DateTime::parse_from_rfc3339(
            value["config"]["anchor"]["pinned_at"]
                .as_str()
                .ok_or("Missing pin time")?,
        )
        .map_err(|e| e.to_string())?;
        ensure(assessed >= pinned, "Assessment predates pinned task")?;
        let mut execution_ids = BTreeSet::new();
        let mut pids = BTreeSet::new();
        for role in ["assessor", "reviewer"] {
            let row = rows
                .get(&format!("worker:{role}"))
                .ok_or("Missing worker effect")?;
            let result = &row["result"];
            let raw = STANDARD
                .decode(
                    result["raw_base64"]
                        .as_str()
                        .ok_or("Missing exact first response")?,
                )
                .map_err(|e| e.to_string())?;
            ensure(
                format!("{:x}", Sha256::digest(&raw)) == result["raw_digest"],
                "First-response hash mismatch",
            )?;
            let decoded = super::parse_json(&raw, 16 * 1024 * 1024)?;
            ensure(
                decoded == result["response"]
                    && result["accepted"] == true
                    && result["execution_mode"] == "software_test",
                "Worker response acceptance mismatch",
            )?;
            let call_id = row["payload"]["call_id"]
                .as_str()
                .ok_or("Missing call ID")?;
            let execution = result["execution_id"]
                .as_str()
                .ok_or("Missing execution ID")?;
            ensure(
                execution.starts_with(&format!("command:{call_id}:")),
                "Execution identity lost call binding",
            )?;
            let evidence = &result["evidence"];
            ensure(
                evidence["returncode"] == 0
                    && evidence["stdout_bytes"] == raw.len()
                    && evidence["call_id"] == call_id
                    && evidence["model"].is_null()
                    && evidence["usage"].is_null()
                    && evidence["receipt_verification"].is_null()
                    && evidence["output_truncated"] == false
                    && row["acknowledged"] == true,
                "Command evidence mismatch",
            )?;
            let pid = evidence["supervisor_pid"]
                .as_u64()
                .filter(|p| *p > 0)
                .ok_or("Missing supervisor identity")?;
            let consumed = row["payload"]["context"]["received_messages"]
                .as_array()
                .ok_or("Missing worker messages")?
                .iter()
                .map(|v| v["message_id"].clone())
                .collect::<Vec<_>>();
            ensure(
                result["response"]["consumed_message_ids"] == json!(consumed),
                "Consumed-message identity mismatch",
            )?;
            execution_ids.insert(execution.to_string());
            pids.insert(pid);
        }
        ensure(
            execution_ids.len() == 2 && pids.len() == 2,
            "Workers shared an execution or supervisor identity",
        )?;
        ensure(
            value["transport"]["usage"]["calls"] == 2
                && value["transport"]["usage"]["messages"] == 2
                && value["transport"]["messages"]
                    .as_array()
                    .ok_or("Missing transport messages")?
                    .iter()
                    .all(|v| v["acked"] == true || v["acked"] == 1),
            "Mailbox delivery accounting mismatch",
        )?;
        let calls = &status["calls"];
        ensure(
            category(calls, "mcp")?.len() == 6 && category(calls, "gateway")?.len() == 2,
            "MCP/gateway call accounting mismatch",
        )?;
        ensure(
            self.command("resume", dir, &[])?["answer"] == expected
                && self.execute(dir)?["answer"] == expected,
            "Replay changed the answer",
        )?;
        let after = self.export(dir, &format!("{label}-replayed"))?;
        ensure(
            after["effects"] == value["effects"]
                && after["status"]["calls"] == *calls
                && after["transport"]["usage"] == value["transport"]["usage"],
            "Replay repeated a call or worker execution",
        )?;
        Ok(
            json!({"transport":value["config"]["transport"]["kind"],"run_id":status["run_id"],"assessment_id":assessment["assessment_id"],"assessed_at":assessment["assessed_at"],"status":status["status"],"collected_count":finding["collected_count"],"reused_count":finding["reused_count"],"worker_executions":execution_ids,"mcp_calls":6,"gateway_calls":2,"replay_added_calls_or_executions":false}),
        )
    }
    fn counts(&self) -> Result<BTreeMap<String, u64>> {
        let connection = rusqlite::Connection::open(self.h.root.join(".eal/runs.sqlite3"))
            .map_err(|e| e.to_string())?;
        let mut statement = connection
            .prepare("SELECT kind,COUNT(*) FROM records GROUP BY kind")
            .map_err(|e| e.to_string())?;
        statement
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
            .map_err(|e| e.to_string())?
            .collect::<std::result::Result<_, _>>()
            .map_err(|e| e.to_string())
    }
    fn unknown(&self) -> Result<Value> {
        let dir = self.h.root.join("unknown-effect");
        let spec = self.spec("unknown-effect", json!({"kind":"json"}))?;
        self.command("start", &dir, &["--spec", &spec.display().to_string()])?;
        let reserved = self.command(
            "reserve",
            &dir,
            &["--role", "assessor", "--execution-mode", "software_test"],
        )?;
        let call_id = reserved["call_id"]
            .as_str()
            .ok_or("Missing reserved call ID")?;
        let before = self.counts()?;
        let database = dir.join("coordination.sqlite");
        let connection = rusqlite::Connection::open(&database).map_err(|e| e.to_string())?;
        connection.execute_batch("CREATE TRIGGER smoke_reject_assessment_receipt BEFORE UPDATE OF state ON effects WHEN NEW.effect_key='assessment' AND NEW.state='committed' BEGIN SELECT RAISE(ABORT,'injected assessment receipt commit failure'); END").map_err(|e|e.to_string())?;
        drop(connection);
        let rejected = self.h.coordinate(
            "query",
            &dir,
            &["--call-id".into(), call_id.into()],
            1,
            true,
        )?;
        ensure(
            rejected["error"]["message"]
                .as_str()
                .is_some_and(|s| s.contains("injected assessment receipt commit failure")),
            "Expected injected receipt failure",
        )?;
        let after = self.counts()?;
        ensure(
            after.get("assessment").copied().unwrap_or(0)
                == before.get("assessment").copied().unwrap_or(0) + 1,
            "Fault injection did not persist exactly one assessment",
        )?;
        let calls = category(&self.command("status", &dir, &[])?["calls"], "mcp")?;
        ensure(calls.len() == 4, "Unexpected calls before unknown recovery")?;
        let connection = rusqlite::Connection::open(&database).map_err(|e| e.to_string())?;
        let row: (String, Option<String>) = connection
            .query_row(
                "SELECT state,result FROM effects WHERE effect_key='assessment'",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .map_err(|e| e.to_string())?;
        ensure(
            row == ("reserved".into(), None),
            "Assessment receipt unexpectedly committed",
        )?;
        connection
            .execute_batch("DROP TRIGGER smoke_reject_assessment_receipt")
            .map_err(|e| e.to_string())?;
        drop(connection);
        let unknown = self.command("query", &dir, &["--call-id", call_id])?;
        ensure(
            unknown["check"]["code"] == "effect_outcome_unknown"
                && unknown["assessments"] == json!({})
                && unknown["packets"] == json!({})
                && unknown["explanations"] == json!({})
                && unknown["collected_count"].is_null()
                && unknown["reused_count"].is_null(),
            "Unknown outcome fabricated assessment data",
        )?;
        ensure(
            self.counts()? == after
                && category(&self.command("status", &dir, &[])?["calls"], "mcp")? == calls,
            "Recovery repeated an uncertain external effect",
        )?;
        let request = &reserved["context"]["received_messages"][0];
        let response = json!({"message":{"schema":"EAL/MAS-coordination/3","kind":"disposition","message_id":"unknown-assessment-handoff","task_id":self.task["task_id"],"task_digest":unknown["task_digest"],"sender":"assessor","recipient":"reviewer","reply_to":request["message_id"],"attempt":0,"finding_id":unknown["finding_id"],"code":"effect_outcome_unknown","outcome":"incomplete"},"consumed_message_ids":[request["message_id"]]});
        let response_path = self.h.root.join("unknown-assessor.json");
        write_json(&response_path, &response)?;
        let raw = super::read_bytes(&response_path, 64 * 1024 * 1024)?;
        let retained = self.command(
            "record",
            &dir,
            &[
                "--call-id",
                call_id,
                "--response",
                &response_path.display().to_string(),
                "--execution-id",
                "software-fixture/injected-assessor",
                "--execution-mode",
                "software_test",
            ],
        )?;
        ensure(
            retained["accepted"] == true
                && STANDARD
                    .decode(
                        retained["raw_base64"]
                            .as_str()
                            .ok_or("Missing retained manual bytes")?,
                    )
                    .map_err(|e| e.to_string())?
                    == raw,
            "Manual assessor first response changed",
        )?;
        let final_status = self.execute(&dir)?;
        let expected = json!({"task_digest":unknown["task_digest"],"finding_id":unknown["finding_id"],"claims":{"works":{"assessment_id":null,"status":null,"assessed_at":null}},"scope":{"context":{"site":"bench"},"requirements":self.task["requirements"]},"conclusion":"unresolved","qualification":"conditional_on_registered_argument"});
        ensure(
            final_status["status"] == "completed_unassessed" && final_status["answer"] == expected,
            "Unknown outcome lost unassessed null fields",
        )?;
        let value = self.export(&dir, "unknown-effect")?;
        let rows = effects(&value)?;
        let reviewer = &rows["worker:reviewer"]["result"];
        ensure(
            rows["assessment"]["status"] == "unknown"
                && rows["assessment"]["result"].is_null()
                && rows["worker:assessor"]["result"]["execution_id"]
                    == "software-fixture/injected-assessor"
                && reviewer["accepted"] == true
                && reviewer["execution_id"]
                    .as_str()
                    .is_some_and(|s| s.starts_with("command:"))
                && reviewer["evidence"]["returncode"] == 0,
            "Unknown disposition worker correspondence failed",
        )?;
        ensure(
            value["transport"]["usage"]["calls"] == 2
                && value["transport"]["usage"]["messages"] == 2
                && self.counts()? == after
                && category(&final_status["calls"], "mcp")? == calls,
            "Unknown outcome recovery changed acquisition accounting",
        )?;
        self.command("resume", &dir, &[])?;
        self.execute(&dir)?;
        let replayed = self.export(&dir, "unknown-effect-replayed")?;
        ensure(
            replayed["effects"] == value["effects"]
                && replayed["status"]["calls"] == value["status"]["calls"]
                && self.counts()? == after,
            "Unknown outcome replay executed another effect",
        )?;
        Ok(
            json!({"status":final_status["status"],"conclusion":final_status["answer"]["conclusion"],"assessment_committed_before_missing_coordination_receipt":true,"additional_assessments":1,"mcp_calls_before_and_after_recovery":calls.len(),"recovery_reassessed":false,"manual_assessor_retained_exactly":true,"reviewer_only_command":reviewer["execution_id"],"all_unassessed_fields_null":true,"resume_added_calls_or_executions":false}),
        )
    }
}
fn effects(value: &Value) -> Result<BTreeMap<String, Value>> {
    value["effects"]
        .as_array()
        .ok_or("Missing coordination effects")?
        .iter()
        .map(|v| {
            Ok((
                v["key"].as_str().ok_or("Missing effect key")?.to_string(),
                v.clone(),
            ))
        })
        .collect()
}
fn category(calls: &Value, name: &str) -> Result<Vec<Value>> {
    Ok(calls
        .as_array()
        .ok_or("Missing calls")?
        .iter()
        .filter(|v| v["category"] == name)
        .cloned()
        .collect())
}
pub fn run(options: &Options) -> Result<Value> {
    let h = Harness::new(
        options,
        "engineering",
        &[
            "eal",
            "eal-coordinate",
            "eal-mcp",
            "eal-mas-server",
            "mas-protocol-codec",
            "eal-process-supervisor",
            "eal-command-supervisor",
            "eal-method-worker",
        ],
    )?;
    let worker = options
        .worker
        .as_ref()
        .ok_or("coordination requires --worker FILE")?
        .canonicalize()
        .map_err(|e| e.to_string())?;
    ensure(worker.is_file(), "Native software worker is missing")?;
    write_json(&h.root.join("worker-environment.json"), &json!({"PATH":""}))?;
    let entry = h.cli(&[
        "register",
        "case.eal",
        "--entry-id",
        "demo",
        "--claim",
        "works",
        "--context",
        "@context.json",
    ])?;
    ensure(
        entry["claims"] == json!(["works"]),
        "Registered claims mismatch",
    )?;
    let task = json!({"task_id":"installed-native-review","question":"Did the supplied test pass?","entry_id":"demo","claims":["works"],"profile":"registered_claims/1","requirements":{"context":{"site":"bench"}}});
    let server = json!({"workspace":h.root,"registry":h.root.join("tools.toml"),"database":null,"methods":"engineering","limits":null});
    let run = Run {
        h: &h,
        worker: worker.clone(),
        entry,
        task,
        server,
    };
    let mut reports = Vec::new();
    for (kind, transport) in [
        ("json", json!({"kind":"json"})),
        ("lab", json!({"kind":"lab","runtime":h.prefix})),
    ] {
        for index in 1..=2 {
            let label = format!("{kind}-{index}");
            let dir = h.root.join(&label);
            let spec = run.spec(&label, transport.clone())?;
            ensure(
                run.command("start", &dir, &["--spec", &spec.display().to_string()])?["status"]
                    == "ready_assessor",
                "Coordination failed to start",
            )?;
            reports.push(run.completed(&dir, &label, u64::from(kind == "json" && index == 1))?);
        }
    }
    let unknown = run.unknown()?;
    h.finish(json!({"worker":worker,"completed_runs":reports,"unknown_effect_recovery":unknown,"direct_native_launches":h.launches.borrow().len()}))
}
