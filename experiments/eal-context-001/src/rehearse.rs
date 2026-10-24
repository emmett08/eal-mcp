//! Synthetic scripted history traces with labelled byte accounting.
use crate::{
    history::{DEFAULT_LIMITS, History, Reference, compact_bytes, exchange},
    save_report, sha256,
};
use serde_json::{Value, json};
use std::path::Path;
const BYTE_CAP: usize = 32_768;
fn synthetic_packet(reference: &Reference, limits: (usize, usize)) -> String {
    json!({"fixture_origin":"synthetic_history_machinery_only","assessment_id":reference.assessment_id,
        "summary_complete":limits.0>=16_384,"omitted":if limits.0<16_384{json!({"details":"synthetic_byte_pressure"})}else{json!({})},
        "fixture_padding":"0123456789abcdef".repeat(if limits.0<16_384{32}else{128})}).to_string()
}
fn trace(condition: char, horizon: usize) -> Result<Value, String> {
    let mut host = History::new(
        condition,
        "Synthetic machinery exercise. Qualify unavailable assessment details.",
        json!({"ready":"synthetic decision identifier"}),
        vec![json!({"name":"synthetic_common_tool","description":"No external or model calls."})],
    )
    .map_err(|e| e.to_string())?;
    let mut anchors: Vec<Reference> = vec![];
    let mut occupancy = vec![];
    let mut transferred = 0;
    let mut missing_historical = 0;
    for turn in 1..=horizon {
        host.begin_turn(&format!("Synthetic history turn {turn}: inspect the scheduled entry and preserve its revision identity.")).map_err(|e|e.to_string())?;
        if turn <= 8 || turn % 10 == 0 {
            let entry = (turn - 1) % 8;
            let reference = Reference {
                entry_id: format!("entry-{entry}"),
                claim: "ready".into(),
                assessment_id: format!("assessment-{turn}"),
                source_digest: format!("source-{entry}-{turn}"),
                context_fingerprint: "context-fixed".into(),
                method_registry_fingerprint: "methods-fixed".into(),
                assessed_at: format!("2040-01-01T00:{:02}:{:02}Z", turn / 60, turn % 60),
            };
            let payload = synthetic_packet(&reference, DEFAULT_LIMITS);
            transferred += payload.len();
            host.add_packet(
                &reference,
                exchange(&format!("anchor-{turn}"), &payload),
                DEFAULT_LIMITS,
            )
            .map_err(|e| e.to_string())?;
            anchors.push(reference);
        }
        if turn % 12 == 0 && !anchors.is_empty() {
            let reference = &anchors[0];
            if condition == 'B' {
                let replay = host.cache_lookup(reference).map_err(|e| e.to_string())?;
                let payload = replay
                    .last()
                    .and_then(|message| message["content"].as_str())
                    .ok_or("cached packet body missing")?;
                transferred += payload.len();
                host.add_packet(
                    reference,
                    exchange(&format!("cache-{turn}"), payload),
                    DEFAULT_LIMITS,
                )
                .map_err(|e| e.to_string())?;
            } else if condition == 'C' {
                let payload = host
                    .reload(reference, |reference, limits| {
                        Ok(synthetic_packet(reference, limits))
                    })
                    .map_err(|e| e.to_string())?;
                transferred += payload.len();
                host.add_packet(
                    reference,
                    exchange(&format!("reload-{turn}-small"), &payload),
                    (4096, 1024),
                )
                .map_err(|e| e.to_string())?;
                if serde_json::from_str::<Value>(&payload).map_err(|e| e.to_string())?["summary_complete"]
                    == false
                {
                    let payload = host
                        .reload(reference, |reference, limits| {
                            Ok(synthetic_packet(reference, limits))
                        })
                        .map_err(|e| e.to_string())?;
                    transferred += payload.len();
                    host.add_packet(
                        reference,
                        exchange(&format!("reload-{turn}-default"), &payload),
                        DEFAULT_LIMITS,
                    )
                    .map_err(|e| e.to_string())?;
                }
            }
        }
        let request = host
            .prepare(|value| Ok(compact_bytes(value)), BYTE_CAP)
            .map_err(|e| e.to_string())?;
        if condition == 'A' && turn % 12 == 0 && !anchors.is_empty() {
            let present = host
                .turns
                .iter()
                .flat_map(|turn| &turn.blocks)
                .any(|block| block.packet && block.reference.as_ref() == Some(&anchors[0]));
            missing_historical += usize::from(!present);
        }
        occupancy.push(compact_bytes(&request));
        host.finish_turn(&format!(
            "Synthetic placeholder answer {turn}; no correctness or retention score is assigned."
        ))
        .map_err(|e| e.to_string())?;
    }
    Ok(
        json!({"condition":condition.to_string(),"scheduled_synthetic_turns":horizon,"completed_synthetic_turns":horizon,"shared_anchor_exposures":anchors.len(),
        "request_size_unit":"UTF-8 bytes of complete serialised synthetic request","request_sampling":"once per synthetic user turn after scripted exchanges; no model request cycles",
        "request_byte_cap":BYTE_CAP,"peak_request_bytes":occupancy.iter().max(),"final_request_bytes":occupancy.last(),"cumulative_serialised_request_bytes":occupancy.iter().sum::<usize>(),
        "logical_historical_lookups":host.logical_lookups,"host_cache_hits":host.cache_hits,"synthetic_service_callback_calls":host.server_reloads,
        "synthetic_packet_body_bytes_exposed":transferred,"A_historical_revisits_without_visible_packet":missing_historical,
        "packet_exchange_evictions":host.events.iter().filter(|event|event["type"]=="packet_eviction").count(),"emergency_turn_evictions":host.events.iter().filter(|event|event["type"]=="emergency_turn_eviction").count(),"events":host.events}),
    )
}
pub fn run(args: &[String]) -> Result<Value, String> {
    let output = match args {
        [] => None,
        [flag, path] if flag == "--output" => Some(Path::new(path)),
        _ => return Err("Usage: eal-context rehearse [--output NEW_FILE]".into()),
    };
    let started = chrono::Utc::now().to_rfc3339();
    let mut traces = vec![];
    for horizon in [40, 80, 120] {
        for condition in ['A', 'B', 'C'] {
            traces.push(trace(condition, horizon)?);
        }
    }
    let result = json!({"status":"synthetic_host_machinery_rehearsal_completed_not_empirical_pilot","protocol_id":"EAL-CONTEXT-001","protocol_version":"0.1.0",
        "implementation":"native Rust history harness/1","command":"eal-context rehearse","started_at_utc":started,"completed_at_utc":chrono::Utc::now().to_rfc3339(),
        "platform":format!("{}-{}",std::env::consts::OS,std::env::consts::ARCH),
        "code_sha256":{"history.rs":sha256(include_bytes!("history.rs")),"history/tests.rs":sha256(include_bytes!("history/tests.rs")),"history/usage.rs":sha256(include_bytes!("history/usage.rs")),"history/budget.rs":sha256(include_bytes!("history/budget.rs")),"rehearse.rs":sha256(include_bytes!("rehearse.rs"))},
        "synthetic_history_traces":traces.len(),"synthetic_turns_completed":720,"model_sessions":0,"provider_model_requests":0,"provider_usage":Value::Null,"quality_scores":Value::Null,
        "limitations":["All packets and turns are synthetic fixtures.","Request occupancy is UTF-8 JSON bytes rather than model tokens.","Request-byte figures use the recorded native JSON implementation.","No model, EAL service or provider request is executed by this rehearsal.","Run cargo test for independent policy assertions; this trace command reports its own exercised scope.","The empirical model-session pilot and independent assessor calibration remain unrun."],"traces":traces});
    if let Some(path) = output {
        save_report(path, &result)?;
    }
    Ok(result)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn nine_traces_complete_and_respect_the_declared_request_byte_cap() {
        let result = run(&[]).unwrap();
        assert_eq!(result["synthetic_history_traces"], 9);
        for trace in result["traces"].as_array().unwrap() {
            assert!(trace["peak_request_bytes"].as_u64().unwrap() <= BYTE_CAP as u64);
        }
        assert_eq!(result["model_sessions"], 0);
        assert_eq!(
            result["traces"]
                .as_array()
                .unwrap()
                .iter()
                .map(|trace| trace["completed_synthetic_turns"].as_u64().unwrap())
                .sum::<u64>(),
            720
        );
    }
}
