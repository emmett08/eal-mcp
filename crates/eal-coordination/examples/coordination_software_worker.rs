//! Deterministic transport fixture: two fresh commands, no model or provider calls.
use eal_core::{ExecutionLimits, json::parse};
use serde_json::{Value, json};
use std::{
    io::{Read, Write},
    process::{Command, ExitCode},
};

fn run() -> Result<Value, String> {
    let mut bytes = Vec::new();
    std::io::stdin()
        .take(4 * 1024 * 1024 + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() > 4 * 1024 * 1024 {
        return Err("Context exceeds 4 MiB".into());
    }
    let limits = ExecutionLimits {
        record_bytes: 16 * 1024 * 1024,
        json_depth: 128,
        ..Default::default()
    };
    let context = parse(
        std::str::from_utf8(&bytes).map_err(|e| e.to_string())?,
        &limits,
    )?;
    let role = context["role"].as_str().ok_or("Missing role")?;
    let argv = context["gateway"]["argv"]
        .as_array()
        .ok_or("Missing gateway argv")?
        .iter()
        .map(|value| value.as_str().ok_or("Invalid gateway argv"))
        .collect::<Result<Vec<_>, _>>()?;
    let (executable, args) = argv.split_first().ok_or("Empty gateway argv")?;
    let received = context["received_messages"]
        .as_array()
        .ok_or("Missing addressed messages")?;
    let mut command = Command::new(executable);
    command.args(args);
    if role == "reviewer" {
        command.args([
            "--finding-id",
            received
                .first()
                .and_then(|message| message["finding_id"].as_str())
                .ok_or("No addressed finding")?,
        ]);
    }
    let output = command.output().map_err(|e| e.to_string())?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).into_owned());
    }
    let finding = parse(
        std::str::from_utf8(&output.stdout).map_err(|e| e.to_string())?,
        &limits,
    )?;
    let consumed = received
        .iter()
        .map(|message| message["message_id"].clone())
        .collect::<Vec<_>>();
    if role == "assessor" {
        let mut message = json!({"schema":"EAL/MAS-coordination/3","kind":"outcome","message_id":"assessor-handoff","task_id":context["task"]["task"]["task_id"],"task_digest":finding["task_digest"],"sender":"assessor","recipient":"reviewer","reply_to":"request","attempt":0,"finding_id":finding["finding_id"]});
        if finding["check"]["ok"] == true {
            let rows = finding["assessments"]
                .as_object()
                .ok_or("Missing assessments")?;
            message["assessment_refs"] = Value::Object(
                rows.iter()
                    .map(|(claim, row)| (claim.clone(), row["assessment_id"].clone()))
                    .collect(),
            );
        } else {
            message["kind"] = json!("disposition");
            message["code"] = finding["check"]["code"].clone();
            message["outcome"] = json!(if ["evidence_unavailable", "effect_outcome_unknown"]
                .contains(&finding["check"]["code"].as_str().unwrap_or(""))
            {
                "incomplete"
            } else {
                "rejected"
            });
        }
        Ok(json!({"message":message,"consumed_message_ids":consumed}))
    } else if role == "reviewer" {
        let task = &context["task"]["task"];
        let claims=task["claims"].as_array().ok_or("Missing selected claims")?.iter().map(|claim|{
            let name=claim.as_str().ok_or("Invalid claim name")?;let row=&finding["assessments"][name];
            Ok((name.to_owned(),json!({"assessment_id":row["assessment_id"],"status":row["status"],"assessed_at":row["assessed_at"]})))
        }).collect::<Result<serde_json::Map<_,_>,String>>()?;
        let conclusion = if finding["check"]["ok"] == true {
            "assessed"
        } else if [
            "source_changed",
            "context_mismatch",
            "workload_mismatch",
            "criteria_mismatch",
        ]
        .contains(&finding["check"]["code"].as_str().unwrap_or(""))
        {
            "out_of_scope"
        } else {
            "unresolved"
        };
        Ok(
            json!({"answer":{"task_digest":finding["task_digest"],"finding_id":finding["finding_id"],"claims":claims,"scope":{"context":task["requirements"]["context"],"requirements":task["requirements"]},"conclusion":conclusion,"qualification":"conditional_on_registered_argument"},"consumed_message_ids":consumed}),
        )
    } else {
        Err("Unknown worker role".into())
    }
}
fn main() -> ExitCode {
    match run() {
        Ok(value) => match serde_json::to_writer(std::io::stdout().lock(), &value) {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                let _ = writeln!(std::io::stderr(), "{error}");
                ExitCode::FAILURE
            }
        },
        Err(error) => {
            let _ = writeln!(std::io::stderr(), "{error}");
            ExitCode::FAILURE
        }
    }
}
