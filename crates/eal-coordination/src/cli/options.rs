use std::{collections::BTreeMap, path::PathBuf};

pub const USAGE: &str = "usage: eal-coordinate COMMAND --run DIRECTORY [OPTIONS]\n\
start (--spec FILE | --demo FILE) [--task-id ID] [--runtime DIRECTORY] [--wall-seconds INTEGER]\n\
status | resume | cancel\n\
reserve --role assessor|reviewer --execution-mode actual_host_agent|provider|software_test [--context-out FILE]\n\
query --call-id ID\n\
lookup --call-id ID --finding-id ID\n\
record --call-id ID --response FILE --execution-id ID --execution-mode MODE\n\
export --output FILE\n\
execute --execution-mode MODE --worker-argv JSON_ARRAY [--worker-environment FILE]\n\
eal-coordinate --response-schema";

pub struct Options {
    pub operation: String,
    pub run: PathBuf,
    fields: BTreeMap<String, String>,
}
impl Options {
    pub fn get(&self, key: &str) -> Option<&str> {
        self.fields.get(key).map(String::as_str)
    }
    pub fn required(&self, key: &str) -> &str {
        self.get(key).expect("required CLI option was checked")
    }
}

pub fn parse(arguments: &[String]) -> Result<Options, String> {
    let operation = arguments.first().ok_or(USAGE)?;
    let (allowed, required): (&[&str], &[&str]) = match operation.as_str() {
        "start" => (
            &[
                "--spec",
                "--demo",
                "--task-id",
                "--runtime",
                "--wall-seconds",
            ],
            &[],
        ),
        "status" | "resume" | "cancel" => (&[], &[]),
        "reserve" => (
            &["--role", "--execution-mode", "--context-out"],
            &["--role", "--execution-mode"],
        ),
        "query" => (&["--call-id"], &["--call-id"]),
        "lookup" => (
            &["--call-id", "--finding-id"],
            &["--call-id", "--finding-id"],
        ),
        "record" => (
            &[
                "--call-id",
                "--response",
                "--execution-id",
                "--execution-mode",
            ],
            &[
                "--call-id",
                "--response",
                "--execution-id",
                "--execution-mode",
            ],
        ),
        "export" => (&["--output"], &["--output"]),
        "execute" => (
            &["--execution-mode", "--worker-argv", "--worker-environment"],
            &["--execution-mode", "--worker-argv"],
        ),
        _ => {
            return Err(format!(
                "Unknown coordination command {operation:?}\n{USAGE}"
            ));
        }
    };
    let mut fields = BTreeMap::new();
    let mut remaining = arguments[1..].iter();
    while let Some(option) = remaining.next() {
        if option != "--run" && !allowed.contains(&option.as_str()) {
            return Err(format!("Unknown option {option:?} for {operation}"));
        }
        let value = remaining
            .next()
            .filter(|value| !value.starts_with("--"))
            .ok_or_else(|| format!("{option} requires a value"))?;
        if fields.insert(option.clone(), value.clone()).is_some() {
            return Err(format!("Repeated option {option}"));
        }
    }
    for key in std::iter::once("--run").chain(required.iter().copied()) {
        if !fields.contains_key(key) {
            return Err(format!("{operation} requires {key}"));
        }
    }
    if operation == "start" {
        if fields.contains_key("--spec") == fields.contains_key("--demo") {
            return Err("start requires exactly one of --spec and --demo".into());
        }
        if fields
            .get("--wall-seconds")
            .is_some_and(|value| value.parse::<i64>().is_err())
        {
            return Err("--wall-seconds requires an integer".into());
        }
    }
    if fields
        .get("--role")
        .is_some_and(|value| !["assessor", "reviewer"].contains(&value.as_str()))
    {
        return Err("--role must be assessor or reviewer".into());
    }
    if fields.get("--execution-mode").is_some_and(|value| {
        !["actual_host_agent", "provider", "software_test"].contains(&value.as_str())
    }) {
        return Err("Unknown worker execution mode".into());
    }
    Ok(Options {
        operation: operation.clone(),
        run: PathBuf::from(&fields["--run"]),
        fields,
    })
}

#[cfg(test)]
mod tests {
    use super::parse;
    fn args(value: &[&str]) -> Vec<String> {
        value.iter().map(|value| (*value).into()).collect()
    }
    #[test]
    fn command_flags_are_closed_and_start_sources_are_exclusive() {
        assert!(parse(&args(&["start", "--run", "r", "--spec", "s"])).is_ok());
        assert!(
            parse(&args(&[
                "start", "--run", "r", "--spec", "s", "--demo", "d"
            ]))
            .is_err()
        );
        assert!(parse(&args(&["status", "--run", "r", "--worker-argv", "[]"])).is_err());
        assert!(
            parse(&args(&[
                "reserve",
                "--run",
                "r",
                "--role",
                "assessor",
                "--execution-mode",
                "software_test"
            ]))
            .is_ok()
        );
        assert!(parse(&args(&["record", "--run", "r", "--call-id", "c"])).is_err());
    }
}
