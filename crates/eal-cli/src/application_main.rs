//! Production EAL operations over the shared native application service.
mod application_commands;
mod application_options;
use std::process::ExitCode;

fn main() -> ExitCode {
    let result = application_options::parse()
        .and_then(|options| options.map(application_commands::run).transpose());
    match result {
        Ok(None) => ExitCode::SUCCESS,
        Ok(Some(value)) => {
            let failure = value.get("valid") == Some(&serde_json::Value::Bool(false))
                || value.get("complete") == Some(&serde_json::Value::Bool(false))
                || value
                    .get("records")
                    .and_then(serde_json::Value::as_object)
                    .is_some_and(|records| {
                        records.values().any(|record| record["status"] == "error")
                    });
            println!(
                "{}",
                serde_json::to_string_pretty(&value).expect("JSON value")
            );
            if failure {
                ExitCode::from(1)
            } else {
                ExitCode::SUCCESS
            }
        }
        Err(message) => {
            println!("{}", serde_json::json!({"error":message}));
            ExitCode::from(2)
        }
    }
}
