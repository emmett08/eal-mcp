//! Actual native CLI/MCP packet diagnostics for explicitly selected revisions.
mod fixtures;
mod runtime;
mod selection;
use crate::{process::execute, save_report, sha256};
use fixtures::{setup, source};
use runtime::{Runtime, normalise_packet, records};
use selection::{binary_hash, options, revision};
use serde_json::{Value, json};
use std::{collections::BTreeMap, fs, path::PathBuf, time::Instant};
const NOW: &str = "2040-01-01T00:00:01Z";
const LATER: &str = "2040-01-01T00:00:02Z";
const SECRET: &str = "NATIVE_PROBE_RAW_OBSERVATION_MUST_STAY_PRIVATE";
const HEADER: &str = "language \"EAL/3\"\nenvironment lab {\n  require site == \"bench\"\n}\ntool reader {\n  version \"1\"\n}\nevidence observation {\n  tool reader\n  kind test\n  environment lab\n  max_age 864000\n  require ok == true\n}\nreasoning authored {\n  method \"structured/1\"\n  rationale \"The calibration predicate supplies conditional authored support.\"\n}\n";
struct Temporary(PathBuf);
impl Temporary {
    fn new() -> Result<Self, String> {
        let path =
            std::env::temp_dir().join(format!("eal-context-native-{}", uuid::Uuid::new_v4()));
        let mut builder = fs::DirBuilder::new();
        #[cfg(unix)]
        {
            use std::os::unix::fs::DirBuilderExt;
            builder.mode(0o700);
        }
        builder.create(&path).map_err(|e| e.to_string())?;
        Ok(Self(path))
    }
}
impl Drop for Temporary {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn ensure(condition: bool, message: &str) -> Result<(), String> {
    if condition {
        Ok(())
    } else {
        Err(message.into())
    }
}
pub fn run(args: &[String]) -> Result<Value, String> {
    let options = options(args)?;
    let started = chrono::Utc::now().to_rfc3339();
    let timer = Instant::now();
    let roots = [
        (
            "reference",
            &options.reference,
            options.reference_revision.as_str(),
        ),
        (
            "candidate",
            &options.candidate,
            options.candidate_revision.as_str(),
        ),
    ];
    let mut hashes = serde_json::Map::new();
    for (name, root, expected) in roots {
        revision(root, expected, &options.git)?;
        if let Some(cargo) = &options.cargo {
            execute(
                cargo,
                &[
                    "build".into(),
                    "--locked".into(),
                    "-p".into(),
                    "eal-cli".into(),
                    "-p".into(),
                    "eal-mcp".into(),
                    "-p".into(),
                    "eal-runtime".into(),
                    "--bins".into(),
                    "--target-dir".into(),
                    root.join("target").into_os_string(),
                ],
                None,
                Some(root),
                false,
                0,
            )?;
        }
        let mut binaries = serde_json::Map::new();
        for binary in [
            "eal",
            "eal-host",
            "eal-mcp",
            "eal-command-supervisor",
            "eal-method-worker",
        ] {
            binaries.insert(
                binary.into(),
                json!(binary_hash(&root.join("target/debug").join(binary))?),
            );
        }
        hashes.insert(name.into(), Value::Object(binaries));
    }
    let mut checks = serde_json::Map::new();
    let mut fixtures = serde_json::Map::new();
    let temporary = Temporary::new()?;
    for kind in ["simple", "byte_pressure", "count_pressure"] {
        let mut assessments = BTreeMap::new();
        for (name, root, _) in roots {
            let workspace = temporary.0.join(format!("{name}-{kind}"));
            setup(&workspace, kind)?;
            let runtime = Runtime::new(root, workspace);
            runtime.register()?;
            assessments.insert(name, runtime.assess(NOW)?);
        }
        let initial = &assessments["candidate"];
        let packet = normalise_packet(&initial["packet"]);
        let parity = packet == normalise_packet(&assessments["reference"]["packet"]);
        ensure(
            parity,
            &format!("{kind} default packet differs from pinned reference"),
        )?;
        checks.insert(format!("{kind}_default_reference_parity"), json!(parity));
        let runtime = Runtime::new(
            &options.candidate,
            temporary.0.join(format!("candidate-{kind}")),
        );
        let id = initial["assessment_id"]
            .as_str()
            .ok_or("assessment ID absent")?;
        let before = records(&runtime.workspace)?;
        let mut ladder = vec![];
        for (bytes, statement) in [(4096, 1024), (16384, 1024), (65536, 4096)] {
            let (retrieved, reading) = runtime.retrieve(id, bytes, statement)?;
            for key in ["assessment_id", "source_digest", "context_fingerprint"] {
                ensure(
                    retrieved[key] == initial[key],
                    &format!("retrieval changed {key}"),
                )?;
            }
            ensure(
                retrieved["assessed_at"] == NOW,
                "retrieval changed original assessment time",
            )?;
            ensure(
                retrieved["packet"]["method_registry_fingerprint"]
                    == initial["packet"]["method_registry_fingerprint"],
                "retrieval changed method identity",
            )?;
            ladder.push(reading);
        }
        let after = records(&runtime.workspace)?;
        ensure(before == after, "historical retrieval created records")?;
        checks.insert(format!("{kind}_retrieval_no_records_added"), json!(true));
        if kind == "byte_pressure" {
            ensure(
                ladder[0]["summary_complete"] == false,
                "small packet failed to report omission",
            )?;
            ensure(
                ladder[2]["summary_complete"] == true,
                "large packet failed to recover statements",
            )?;
            checks.insert("byte_pressure_4096_reports_omission".into(), json!(true));
            checks.insert(
                "byte_pressure_65536_recovers_statements".into(),
                json!(true),
            );
        }
        if kind == "count_pressure" {
            ensure(
                ladder[2]["summary_complete"] == false,
                "count pressure failed to remain explicit",
            )?;
            checks.insert(
                "count_pressure_remains_incomplete_at_65536".into(),
                json!(true),
            );
        }
        let mut fixture = json!({"source_sha256":sha256(source(kind)?.as_bytes()),"normalised_default_packet_sha256":sha256(packet.to_string().as_bytes()),"retrieval_ladder":ladder,"records_before_retrieval":before,"records_after_retrieval":after});
        if kind == "simple" {
            fs::write(
                runtime.workspace.join("case.eal"),
                source(kind)?.replace(
                    "The calibrated service can proceed",
                    "The revised calibrated service can proceed",
                ),
            )
            .map_err(|e| e.to_string())?;
            let later = runtime.assess(LATER)?;
            let (historical, reading) = runtime.retrieve(id, 16384, 1024)?;
            ensure(
                later["source_digest"] != initial["source_digest"]
                    && later["assessment_id"] != initial["assessment_id"],
                "source edit failed to create a new assessment",
            )?;
            ensure(
                historical["source_digest"] == initial["source_digest"]
                    && historical["assessed_at"] == NOW
                    && historical["packet"] == initial["packet"],
                "historical retrieval changed revision or time",
            )?;
            checks.insert("source_edit_creates_new_assessment".into(), json!(true));
            checks.insert(
                "historical_retrieval_preserves_original_revision_and_time".into(),
                json!(true),
            );
            fixture["historical_retrieval_after_edit"] = reading;
            fs::copy(
                runtime.workspace.join("case.eal"),
                runtime.workspace.join("unexposed.eal"),
            )
            .map_err(|e| e.to_string())?;
            runtime.cli(&[
                "register",
                "unexposed.eal",
                "--entry-id",
                "unexposed",
                "--claim",
                "ready",
                "--context",
                "{\"site\":\"bench\"}",
            ])?;
            let (rejected,_)=runtime.host(&json!({"operation":"get_known_packet","entry_id":"unexposed","assessment_id":later["assessment_id"],"claim":"ready"}),1)?;
            ensure(
                rejected["is_error"] == true,
                "registered allowlist failed to reject unexposed entry",
            )?;
            checks.insert("registered_allowlist_rejection".into(), json!(true));
            fixture["unexposed_entry_rejection"] = rejected;
            let prepared = runtime.cli(&[
                "model-context",
                "demo",
                "--claim",
                "ready",
                "--now",
                LATER,
                "--question",
                "What does this calibrated evidence establish?",
            ])?;
            ensure(
                prepared["messages"]
                    .as_array()
                    .is_some_and(|messages| !messages.is_empty()),
                "native model context has no messages",
            )?;
            ensure(
                !prepared.to_string().contains(SECRET),
                "raw observation appeared in model context",
            )?;
            checks.insert(
                "native_model_context_messages_available".into(),
                json!(true),
            );
            checks.insert("raw_observation_secret_excluded".into(), json!(true));
            fixture["model_context_messages_sha256"] =
                json!(sha256(prepared["messages"].to_string().as_bytes()));
        }
        fixtures.insert(kind.into(), fixture);
    }
    let report = json!({"status":"native_machinery_diagnostic_only","protocol":"EAL-CONTEXT-001","implementation":"Rust packet diagnostic/1","harness_sha256":sha256(&[include_bytes!("mod.rs").as_slice(),include_bytes!("runtime.rs").as_slice(),include_bytes!("fixtures.rs").as_slice(),include_bytes!("selection.rs").as_slice(),include_bytes!("../process.rs").as_slice()].concat()),"invocation":std::env::args().collect::<Vec<_>>(),"reference_sha":options.reference_revision,"candidate_sha":options.candidate_revision,"binary_sha256":hashes,"built_pinned_checkouts":options.cargo.is_some(),"started_at_utc":started,"finished_at_utc":chrono::Utc::now().to_rfc3339(),"elapsed_seconds":timer.elapsed().as_secs_f64(),"child_path":"","model_sessions":0,"provider_usage":Value::Null,"provider_usage_reason":"No language model or provider endpoint is invoked by this diagnostic.","limitations":["Hand-constructed fixtures calibrate native packet machinery only.","The 144-session model pilot remains unrun.","Default parity normalises assessment, collection and observation IDs only.","The reference executable supplies packet compatibility rather than a timed workflow comparison.","Binary digests identify the supplied executables; use --build to rebuild the verified clean pinned checkouts."],"checks_passed":checks.len(),"checks_failed":0,"checks":checks,"fixtures":fixtures});
    save_report(&options.output, &report)?;
    Ok(report)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn normalisation_preserves_repeated_identity_relations_and_other_content() {
        let packet = json!({"assessment_id":"a","collection_id":"c","evidence":{"x":{"observation_id":"o"}},"repeated":["a","c","o","unrelated"],"assessed_at":NOW});
        let normal = normalise_packet(&packet);
        assert_eq!(
            normal["repeated"],
            json!([
                "<ASSESSMENT>",
                "<COLLECTION>",
                "<OBSERVATION:x>",
                "unrelated"
            ])
        );
        assert_eq!(normal["assessed_at"], NOW);
    }
    #[test]
    fn authored_pressure_cases_preserve_the_declared_fixture_sizes() {
        assert!(source("simple").unwrap().contains("argument route ="));
        assert_eq!(
            source("count_pressure")
                .unwrap()
                .matches("argument route")
                .count(),
            40
        );
        assert_eq!(
            source("byte_pressure")
                .unwrap()
                .matches("claim premise")
                .count(),
            6
        );
        assert!(source("unknown").is_err());
    }
    #[test]
    fn revision_and_report_options_are_explicit() {
        assert!(options(&[]).is_err());
        assert!(options(&["--candidate-revision".into(), "invalid".into()]).is_err());
    }
}
