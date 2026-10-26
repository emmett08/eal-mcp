//! Closed CLI operations over a durable ticket journal.
use super::{artifacts, journal::Journal, run};
use crate::host::HostPaths;
use eal_coordination_contracts::ticket::TicketSpec;
use eal_runtime::private_sqlite::ensure_immutable_file;
use serde_json::json;
use std::{path::PathBuf, process::ExitCode};

pub fn command(arguments: &[String]) -> Option<ExitCode> {
    let operation = arguments.first()?.as_str();
    if ![
        "ticket-run",
        "ticket-status",
        "ticket-export",
        "ticket-cancel",
    ]
    .contains(&operation)
    {
        return None;
    }
    let result = (|| {
        let mut directory = None;
        let mut spec = None;
        let mut output = None;
        let mut args = arguments[1..].iter();
        while let Some(option) = args.next() {
            let selected = match option.as_str() {
                "--run" => &mut directory,
                "--spec" if operation == "ticket-run" => &mut spec,
                "--output" if operation == "ticket-export" => &mut output,
                _ => return Err("unknown ticket command option".into()),
            };
            let value = args
                .next()
                .filter(|s| !s.starts_with("--"))
                .ok_or("ticket option requires a path")?;
            if selected.replace(PathBuf::from(value)).is_some() {
                return Err("repeated ticket option".into());
            }
        }
        let directory = directory.ok_or("ticket commands require --run DIRECTORY")?;
        let journal = if operation == "ticket-run" {
            let path = spec
                .ok_or("ticket-run requires --spec FILE")?
                .canonicalize()
                .map_err(|e| e.to_string())?;
            let bytes = artifacts::read(&path, 4 * 1024 * 1024)?;
            let spec: TicketSpec = serde_json::from_value(artifacts::json(&bytes)?)
                .map_err(|_| "invalid ticket specification")?;
            run::validate_spec(&spec)?;
            if directory.join("ticket.sqlite").exists() {
                let existing = Journal::open(&directory)?;
                if existing.spec != spec {
                    return Err("ticket resume must use the exact frozen specification".into());
                }
                existing
            } else {
                Journal::create(&directory, spec)?
            }
        } else {
            Journal::open(&directory)?
        };
        let result = match operation {
            "ticket-run" => run::execute(&journal, &HostPaths::current()?)?,
            "ticket-cancel" => {
                journal.state.cancel()?;
                run::snapshot(&journal)?
            }
            _ => run::snapshot(&journal)?,
        };
        if let Some(path) = output {
            ensure_immutable_file(
                path,
                &serde_json::to_vec_pretty(&result).map_err(|e| e.to_string())?,
            )?;
        }
        Ok::<_, String>(result)
    })();
    Some(match result {
        Ok(value) => {
            println!(
                "{}",
                serde_json::to_string_pretty(&value).expect("finite ticket status")
            );
            if operation == "ticket-run" && value["status"] != "completed" {
                ExitCode::from(1)
            } else {
                ExitCode::SUCCESS
            }
        }
        Err(error) => {
            let message = error.chars().take(512).collect::<String>();
            eprintln!(
                "{}",
                json!({"error":{"type":"TicketError","message":message}})
            );
            ExitCode::from(1)
        }
    })
}
