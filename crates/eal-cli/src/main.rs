//! Run the Rust language front end without a model, server or source resolver.

use std::io::{self, Read, Write};
use std::process::ExitCode;

const MAX_SOURCE_BYTES: u64 = 1024 * 1024;
const USAGE: &str = "usage: eal-syntax <recognise|parse|format> <FILE|->\n\
    recognise  Emit the generated grammar's owned syntax tree as JSON\n\
    parse      Emit the structurally checked, unexpanded lexical AST as JSON\n\
    format     Emit canonical source from the checked lexical AST\n\
    -          Read UTF-8 source from standard input\n\
Source imports remain unresolved. Binding and evaluation are later port stages.";

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err((code, message)) => {
            eprintln!("{message}");
            ExitCode::from(code)
        }
    }
}

fn run() -> Result<(), (u8, String)> {
    let arguments: Vec<_> = std::env::args_os().skip(1).collect();
    if arguments.len() == 1 && matches!(arguments[0].to_str(), Some("--help" | "-h")) {
        println!("{USAGE}");
        return Ok(());
    }
    if arguments.len() != 2 {
        return Err((2, USAGE.into()));
    }
    let command = arguments[0].to_str().unwrap_or("");
    if !matches!(command, "recognise" | "parse" | "format") {
        return Err((2, USAGE.into()));
    }
    let mut bytes = Vec::new();
    if arguments[1] == "-" {
        io::stdin()
            .take(MAX_SOURCE_BYTES + 1)
            .read_to_end(&mut bytes)
            .map_err(|error| (2, error.to_string()))?;
    } else {
        // Resolve an operator-selected source alias once, then retain that target.
        let path = std::path::Path::new(&arguments[1])
            .canonicalize()
            .map_err(|error| (2, error.to_string()))?;
        if std::fs::metadata(&path)
            .map_err(|error| (2, error.to_string()))?
            .len()
            > MAX_SOURCE_BYTES
        {
            return Err((1, "source exceeds the 1 MiB input limit".into()));
        }
        bytes = eal_runtime::acquisition::read_regular(
            &path,
            MAX_SOURCE_BYTES as usize,
            "Source must be a regular file",
        )
        .map_err(|error| (2, error.message))?;
    }
    if bytes.len() as u64 > MAX_SOURCE_BYTES {
        return Err((1, "source exceeds the 1 MiB input limit".into()));
    }
    let source = String::from_utf8(bytes).map_err(|error| (2, error.to_string()))?;
    let stdout = io::stdout();
    let mut output = stdout.lock();
    match command {
        "recognise" => {
            let document = eal_parser::parse(&source).map_err(|error| (1, error.to_string()))?;
            serde_json::to_writer_pretty(&mut output, &document)
                .map_err(|error| (2, error.to_string()))?;
            writeln!(output).map_err(|error| (2, error.to_string()))?;
        }
        "parse" => {
            let program = eal_core::parse(&source).map_err(|error| (1, error.to_string()))?;
            serde_json::to_writer_pretty(&mut output, &program)
                .map_err(|error| (2, error.to_string()))?;
            writeln!(output).map_err(|error| (2, error.to_string()))?;
        }
        "format" => {
            let program = eal_core::parse(&source).map_err(|error| (1, error.to_string()))?;
            let formatted = program
                .format_with_limits(eal_core::ParseLimits::default())
                .map_err(|error| (1, error.to_string()))?;
            output
                .write_all(formatted.as_bytes())
                .map_err(|error| (2, error.to_string()))?;
        }
        _ => unreachable!("command checked above"),
    }
    Ok(())
}
