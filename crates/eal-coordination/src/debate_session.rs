//! Coordinator-owned graph routing for an external host using typed JSON lines.
//!
//! This session executes opaque application payloads independently of the
//! registered-argument assessor/reviewer profile. Reserved unknown calls are
//! retained for inspection and never automatically executed again.
mod io;
mod journal;

use eal_coordination_contracts::{ClientMessage, MAX_FRAME_BYTES, ServerMessage, Session};
use serde_json::Value;
use std::{
    io::{BufRead, Write},
    path::{Path, PathBuf},
    process::ExitCode,
};

/// Intercept only the separate debate commands, leaving registered coordination unchanged.
pub fn command(arguments: &[String]) -> Option<ExitCode> {
    let operation = arguments.first()?.as_str();
    if !["debate-session", "debate-status", "debate-export"].contains(&operation) {
        return None;
    }
    let result = (|| -> Result<(), String> {
        let (run, output) = options(&arguments[1..], operation == "debate-export")?;
        match operation {
            "debate-session" => run_session(
                &run,
                &mut std::io::stdin().lock(),
                &mut std::io::stdout().lock(),
            ),
            "debate-status" | "debate-export" => {
                let value = if operation == "debate-status" {
                    journal::status(&run)?
                } else {
                    journal::export(&run, output.as_deref())?
                };
                println!(
                    "{}",
                    serde_json::to_string_pretty(&value).map_err(|error| error.to_string())?
                );
                Ok(())
            }
            _ => unreachable!(),
        }
    })();
    Some(match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!(
                "{}",
                serde_json::json!({"error":{"type":"DebateSessionError","message":error}})
            );
            ExitCode::from(1)
        }
    })
}
fn options(arguments: &[String], allow_output: bool) -> Result<(PathBuf, Option<PathBuf>), String> {
    let mut run = None;
    let mut output = None;
    let mut remaining = arguments.iter();
    while let Some(option) = remaining.next() {
        let selected = match option.as_str() {
            "--run" => &mut run,
            "--output" if allow_output => &mut output,
            _ => {
                return Err(
                    "debate commands accept --run and debate-export also accepts --output".into(),
                );
            }
        };
        let value = remaining
            .next()
            .filter(|value| !value.starts_with("--"))
            .ok_or("debate option requires a path")?;
        if selected.replace(PathBuf::from(value)).is_some() {
            return Err("repeated debate option".into());
        }
    }
    Ok((
        run.ok_or("debate commands require --run DIRECTORY")?,
        output,
    ))
}
fn decode(bytes: &[u8]) -> Result<ClientMessage, String> {
    let value = eal_collector_contracts::strict_json_exact(
        bytes,
        eal_collector_contracts::JsonLimits {
            max_bytes: MAX_FRAME_BYTES,
            max_depth: 128,
            max_nodes: MAX_FRAME_BYTES,
        },
    )?;
    serde_json::from_value(value).map_err(|error| format!("invalid session frame: {error}"))
}
fn read<R: BufRead>(
    journal: &mut journal::Journal,
    input: &mut R,
) -> Result<(ClientMessage, Value), String> {
    match io::read_frame(input, MAX_FRAME_BYTES) {
        Ok(Some(bytes)) => {
            let frame = journal.frame("in", &bytes, true)?;
            Ok((decode(&bytes)?, frame))
        }
        Ok(None) => Err("external host closed input before completing the session".into()),
        Err(error) => {
            journal.frame("in", &error.bytes, false)?;
            Err(error.message)
        }
    }
}
fn emit<W: Write>(
    journal: &mut journal::Journal,
    output: &mut W,
    message: &ServerMessage,
) -> Result<Value, String> {
    let mut bytes = serde_json::to_vec(message).map_err(|error| error.to_string())?;
    bytes.push(b'\n');
    if bytes.len() > MAX_FRAME_BYTES {
        return Err("server session frame exceeds its byte bound".into());
    }
    let frame = journal.frame("out", &bytes, true)?;
    output
        .write_all(&bytes)
        .and_then(|()| output.flush())
        .map_err(|error| error.to_string())?;
    Ok(frame)
}
fn execute<R: BufRead, W: Write>(
    journal: &mut journal::Journal,
    input: &mut R,
    output: &mut W,
) -> Result<(), String> {
    let (message, _) = read(journal, input)?;
    let ClientMessage::Start { spec } = message else {
        return Err("first session frame must be start".into());
    };
    let mut session = Session::new(spec.clone()).map_err(|error| error.to_string())?;
    journal.freeze(&spec, session.total_calls())?;
    while !session.done() {
        let dispatch = session.dispatch().map_err(|error| error.to_string())?;
        // Retain the exact dispatch and reserve every call before its frame is published.
        let message = ServerMessage::Dispatch {
            dispatch: dispatch.clone(),
        };
        let mut bytes = serde_json::to_vec(&message).map_err(|error| error.to_string())?;
        bytes.push(b'\n');
        if bytes.len() > MAX_FRAME_BYTES {
            return Err("server session frame exceeds its byte bound".into());
        }
        let frame = journal.frame("out", &bytes, true)?;
        journal.reserve(&dispatch, &frame)?;
        output
            .write_all(&bytes)
            .and_then(|()| output.flush())
            .map_err(|error| error.to_string())?;
        let (message, frame) = read(journal, input)?;
        let ClientMessage::Results { result } = message else {
            return Err("a dispatched round requires results".into());
        };
        let responses = result.responses.clone();
        session.accept(result).map_err(|error| error.to_string())?;
        journal.accept(&responses, &frame)?;
    }
    journal.completed(session.rounds(), session.total_calls())?;
    emit(
        journal,
        output,
        &ServerMessage::Completed {
            rounds: session.rounds(),
            calls: session.total_calls(),
        },
    )?;
    Ok(())
}
fn run_session<R: BufRead, W: Write>(
    directory: &Path,
    input: &mut R,
    output: &mut W,
) -> Result<(), String> {
    let mut journal = journal::Journal::new(directory)?;
    if let Err(error) = execute(&mut journal, input, output) {
        let diagnostic = journal.failed(&error)?;
        let _ = emit(
            &mut journal,
            output,
            &ServerMessage::Failed {
                message: diagnostic.clone(),
            },
        );
        return Err(diagnostic);
    }
    Ok(())
}
