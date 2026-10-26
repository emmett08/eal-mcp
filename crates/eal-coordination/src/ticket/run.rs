//! Stage admission, supervised execution and terminal ticket-state derivation.
use super::{
    artifacts,
    journal::{Journal, key},
};
use crate::{
    host::HostPaths,
    ledger::EffectStatus,
    workers::{CommandWorker, CommandWorkerConfig, WorkerError},
};
use eal_coordination_contracts::ticket::{
    Artifact, STAGE_REQUEST_SCHEMA, STAGE_RESULT_SCHEMA, Stage, StageRequest, StageResult,
    StageStatus, TICKET_SCHEMA, TicketSpec,
};
use eal_runtime::process::Supervisor;
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
    time::Duration,
};

pub(super) fn validate_spec(spec: &TicketSpec) -> Result<(), String> {
    if serde_json::to_vec(&spec.context)
        .map_err(|e| e.to_string())?
        .len()
        > 65536
    {
        return Err("ticket application context exceeds its 65536-byte bound; retain larger input as an artifact".into());
    }
    if spec.schema_version != TICKET_SCHEMA
        || spec.ticket_id.is_empty()
        || spec.ticket_id.len() > 80
        || !spec
            .ticket_id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"_.-".contains(&b))
    {
        return Err("ticket requires its supported schema and bounded identifier".into());
    }
    if !spec.workspace.is_absolute()
        || !spec.workspace.is_dir()
        || spec.workspace.canonicalize().map_err(|e| e.to_string())? != spec.workspace
    {
        return Err("ticket workspace must be an existing canonical directory".into());
    }
    if spec.worker_argv.is_empty()
        || spec.worker_argv.len() > 64
        || spec
            .worker_argv
            .iter()
            .any(|a| a.is_empty() || a.len() > 4096 || a.contains('\0'))
        || !Path::new(&spec.worker_argv[0]).is_absolute()
    {
        return Err("ticket worker requires a bounded argv and absolute executable".into());
    }
    if !(1..=86400).contains(&spec.timeout_seconds)
        || !(1..=3600).contains(&spec.stage_timeout_seconds)
        || spec.stage_timeout_seconds > spec.timeout_seconds
    {
        return Err("ticket deadlines require 1..86400 total seconds and 1..3600 stage seconds within the total".into());
    }
    eal_collector_contracts::validate_environment_names(&spec.environment, 64)?;
    if spec.environment.iter().collect::<BTreeSet<_>>().len() != spec.environment.len() {
        return Err("ticket environment names must be distinct".into());
    }
    artifacts::validate(&spec.workspace, &spec.inputs)
}

pub(super) fn validate_result(
    spec: &TicketSpec,
    stage: Stage,
    result: &StageResult,
    previous: &[Artifact],
) -> Result<(), String> {
    if serde_json::to_vec(&result.outcome)
        .map_err(|e| e.to_string())?
        .len()
        > 65536
    {
        return Err(
            "ticket stage outcome exceeds its 65536-byte bound; retain larger results as artifacts"
                .into(),
        );
    }
    if result.schema_version != STAGE_RESULT_SCHEMA
        || result.ticket_id != spec.ticket_id
        || result.stage != stage
    {
        return Err("worker response does not identify its dispatched ticket stage".into());
    }
    artifacts::validate(&spec.workspace, &result.artifacts)?;
    if result.status == StageStatus::Completed {
        for kind in stage.required_artifacts() {
            if !result
                .artifacts
                .iter()
                .any(|artifact| artifact.kind == *kind)
            {
                return Err(format!(
                    "completed {} stage lacks required {kind} artifact",
                    stage.as_str()
                ));
            }
        }
        if stage == Stage::Execute {
            let outcome = result
                .outcome
                .as_ref()
                .ok_or("execution stage requires an assessment outcome")?;
            if outcome["complete"] != true
                || outcome["valid"] != true
                || !outcome["claim_status"].is_string()
            {
                return Err("completed execution requires a valid complete assessment and explicit claim status".into());
            }
            super::execution::validate(result, previous)?;
        }
    }
    Ok(())
}

struct Progress {
    artifacts: Vec<Artifact>,
    terminal: Option<String>,
    outcome: Option<Value>,
    effects: usize,
}

fn prior(journal: &Journal) -> Result<Progress, String> {
    let mut artifacts = Vec::new();
    let mut outcome = None;
    let mut status = None;
    let mut effects = 0;
    let mut executions = BTreeSet::new();
    for stage in Stage::ALL {
        let Some(effect) = journal.state.effect(&key(stage))? else {
            break;
        };
        effects += 1;
        if effect["payload"]["previous"]
            != serde_json::to_value(&artifacts).map_err(|e| e.to_string())?
        {
            return Err(
                "ticket stage dependency receipts differ from earlier committed artifacts".into(),
            );
        }
        if effect["status"] != "committed" {
            status = Some("uncertain".into());
            break;
        }
        if effect["result"]["interrupted"] == true {
            journal.response(stage, &effect)?;
            status = Some("failed".into());
            break;
        }
        if effect["result"]["rejected"] == true {
            journal.response(stage, &effect)?;
            status = Some("failed".into());
            break;
        }
        let response = journal
            .response(stage, &effect)?
            .ok_or("committed stage has no complete response")?;
        let execution = effect["result"]["execution_id"]
            .as_str()
            .ok_or("accepted ticket stage lacks an execution identity")?;
        if !executions.insert(execution.to_owned()) {
            return Err("ticket stages share an execution identity".into());
        }
        artifacts.extend(response.artifacts);
        artifacts::validate(&journal.spec.workspace, &artifacts)?;
        if response.status == StageStatus::Blocked {
            status = Some("blocked".into());
            outcome = response.outcome;
            break;
        }
        outcome = response.outcome;
    }
    Ok(Progress {
        artifacts,
        terminal: status,
        outcome,
        effects,
    })
}

pub(super) fn snapshot(journal: &Journal) -> Result<Value, String> {
    validate_spec(&journal.spec)?;
    let progress = prior(journal)?;
    let Progress {
        artifacts,
        terminal,
        outcome,
        effects: observed_effects,
    } = progress;
    let effects = journal.state.effects()?;
    if effects.len() != observed_effects
        || effects.len() > Stage::ALL.len()
        || effects
            .iter()
            .zip(Stage::ALL)
            .any(|(effect, stage)| effect["key"] != key(stage))
    {
        return Err("ticket contains an unexpected or out-of-order stage effect".into());
    }
    let mut status = terminal.unwrap_or_else(|| {
        if effects.len() == Stage::ALL.len() {
            "completed"
        } else {
            "ready"
        }
        .into()
    });
    if status != "completed" && journal.state.cancelled()? {
        status = "cancelled".into();
    } else if status == "ready" && remaining(journal)?.is_zero() {
        status = "expired".into();
    }
    Ok(
        json!({"schema":"EAL/ticket-status/1","ticket_id":journal.spec.ticket_id,"status":status,"artifacts":artifacts,"outcome":outcome,"stages":effects,"automatic_reexecution":false}),
    )
}

pub(super) fn execute(journal: &Journal, host: &HostPaths) -> Result<Value, String> {
    validate_spec(&journal.spec)?;
    let current = snapshot(journal)?;
    if current["status"] != "ready" {
        return Ok(current);
    }
    let environment = journal
        .spec
        .environment
        .iter()
        .map(|name| {
            let value = std::env::var(name).map_err(|_| {
                format!("configured ticket environment name is unavailable: {name}")
            })?;
            Ok((name.clone(), value))
        })
        .collect::<Result<BTreeMap<_, _>, String>>()?;
    let worker = CommandWorker::new(
        Supervisor::new(host.supervisor.clone()),
        CommandWorkerConfig {
            argv: journal.spec.worker_argv.clone(),
            execution_mode: "actual_host_agent".into(),
            timeout_seconds: journal.spec.stage_timeout_seconds,
            max_output_bytes: 4 * 1024 * 1024,
            environment,
            working_directory: journal.spec.workspace.clone(),
            lease_directory: journal.directory.clone(),
        },
    )?;
    for stage in Stage::ALL {
        let Progress {
            artifacts: previous,
            terminal,
            ..
        } = prior(journal)?;
        if terminal.is_some() || journal.state.cancelled()? {
            return snapshot(journal);
        }
        if journal.state.effect(&key(stage))?.is_some() {
            continue;
        }
        artifacts::validate(&journal.spec.workspace, &journal.spec.inputs)?;
        journal.check_worker()?;
        let remaining = remaining(journal)?;
        if remaining.is_zero() {
            journal
                .state
                .event("ticket_expired", &json!({"stage":stage}))?;
            return snapshot(journal);
        }
        let call_id = format!("ticket:{}:{}", journal.spec.ticket_id, stage.as_str());
        let request = StageRequest {
            schema_version: STAGE_REQUEST_SCHEMA.into(),
            ticket_id: journal.spec.ticket_id.clone(),
            stage,
            workspace: journal.spec.workspace.clone(),
            context: journal.spec.context.clone(),
            previous,
            call_id: call_id.clone(),
        };
        let request = serde_json::to_value(request).map_err(|e| e.to_string())?;
        let reservation = journal.request(stage, &request)?;
        if journal
            .state
            .reserve_effect(&key(stage), &reservation)?
            .status
            != EffectStatus::New
        {
            return snapshot(journal);
        }
        let cancellation = || journal.state.cancelled();
        let result = match worker.execute_exact(&request, &call_id, remaining, Some(&cancellation))
        {
            Ok(output) => {
                let raw = journal.raw_response(stage, &output.raw_response)?;
                let evidence_admitted = serde_json::to_vec(&output.evidence)
                    .map_err(|e| e.to_string())?
                    .len()
                    <= 65536;
                let evidence = if evidence_admitted {
                    output.evidence
                } else {
                    json!({"diagnostic":"execution_evidence_exceeds_byte_bound"})
                };
                let admitted = (|| {
                    if !evidence_admitted {
                        return Err("execution evidence exceeds its byte bound".into());
                    }
                    if journal.state.effects()?.iter().any(|effect| {
                        effect["status"] == "committed"
                            && effect["result"]["execution_id"] == output.execution_id
                    }) {
                        return Err("ticket stages require distinct execution identities".into());
                    }
                    let response: StageResult = serde_json::from_value(output.response.clone())
                        .map_err(|e| e.to_string())?;
                    let previous: Vec<Artifact> =
                        serde_json::from_value(request["previous"].clone())
                            .map_err(|e| e.to_string())?;
                    validate_result(&journal.spec, stage, &response, &previous)?;
                    Ok::<_, String>(response)
                })();
                match admitted {
                    Ok(response) => {
                        json!({"raw":raw,"response_sha256":artifacts::digest(&serde_json::to_vec(&response).map_err(|e|e.to_string())?),"execution_id":output.execution_id,"evidence":evidence,"interrupted":false,"rejected":false})
                    }
                    Err(_) => {
                        json!({"raw":raw,"response":null,"execution_id":output.execution_id,"evidence":evidence,"interrupted":false,"rejected":true,"diagnostic":"stage_response_rejected"})
                    }
                }
            }
            Err(WorkerError::Interrupted(error)) => {
                let raw = journal.raw_response(stage, &error.raw_response)?;
                json!({"raw":raw,"response":null,"execution_id":error.execution_id,"evidence":error.evidence,"interrupted":true,"rejected":false,"diagnostic":error.reason})
            }
            Err(WorkerError::Invalid(_)) => {
                let raw = journal.raw_response(stage, &[])?;
                json!({"raw":raw,"response":null,"execution_id":null,"evidence":{},"interrupted":true,"rejected":false,"diagnostic":"worker_admission_failed"})
            }
        };
        journal.state.complete_effect(&key(stage), &result, &[])?;
        journal.state.event("ticket_stage_recorded",&json!({"stage":stage,"rejected":result["rejected"],"interrupted":result["interrupted"]}))?;
    }
    snapshot(journal)
}

fn remaining(journal: &Journal) -> Result<Duration, String> {
    let config = journal.state.config()?;
    let deadline = config["deadline_unix_ms"]
        .as_u64()
        .ok_or("ticket deadline is missing")?;
    Ok(Duration::from_millis(
        deadline.saturating_sub(super::journal::unix_ms()?),
    ))
}
