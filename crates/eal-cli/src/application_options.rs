//! Validate production CLI option scope before constructing application state.
use eal_runtime::bootstrap::RuntimeOptions;
use std::collections::BTreeMap;

pub const USAGE: &str = "usage: eal [--workspace DIR] [--registry FILE] [--database FILE]\n\
    [--limits FILE] [--methods builtin|engineering|aspic|all] COMMAND\n\
describe\nvalidate SOURCE\nformat SOURCE\nplan SOURCE --claim NAME\n\
collect SOURCE --context JSON|@FILE [--claim NAME | --evidence NAME ...]\n\
reason SOURCE --context JSON|@FILE [--collection ID] [--now ISO_TIME]\n\
compile-aspic SOURCE --context JSON|@FILE --collection ID --goal NAME\n\
explain ASSESSMENT_ID [--claim NAME]\npacket ASSESSMENT_ID [--claim NAME]\n\
register SOURCE [--entry-id ID] [--label TEXT] [--claim NAME ...] [--context JSON|@FILE]\n\
register-tree [DIRECTORY] [--context JSON|@FILE] [--limit N]\n\
sources [--limit N] [--offset N]\n\
find [QUERY] [--claim NAME] [--context JSON|@FILE] [--path SOURCE] [--limit N]\n\
assess-known ENTRY_ID --claim NAME [--context JSON|@FILE] [--now ISO_TIME] [--reuse compatible|fresh]\n\
history ENTRY_ID [--source-digest SHA256] [--limit N]\n\
model-context ENTRY_ID --claim NAME --question TEXT [--context JSON|@FILE] [--now ISO_TIME] [--reuse compatible|fresh]\n\
grounded GRAPH_JSON\nexport-aspic RESULT_JSON --output FILE\n\
compile-aspic additionally accepts --semantics, --query-mode and --preference.\n\
Native companions default to the executable directory; --supervisor,\n\
--method-worker and --workers configure their operator-owned runtime.";

pub struct Options {
    pub runtime: RuntimeOptions,
    pub command: String,
    pub positional: Option<String>,
    fields: BTreeMap<String, Vec<String>>,
}
impl Options {
    pub fn value(&self, name: &str) -> Option<&str> {
        self.fields
            .get(name)
            .and_then(|v| v.first())
            .map(String::as_str)
    }
    pub fn values(&self, name: &str) -> Option<&[String]> {
        self.fields.get(name).map(Vec::as_slice)
    }
    pub fn required(&self, name: &str) -> Result<&str, String> {
        self.value(name)
            .ok_or_else(|| format!("{} requires {name}", self.command))
    }
    pub fn number(&self, name: &str, default: usize) -> Result<usize, String> {
        self.value(name)
            .map(|value| {
                value
                    .parse()
                    .map_err(|_| format!("{name} requires an integer"))
            })
            .unwrap_or(Ok(default))
    }
}

pub fn parse() -> Result<Option<Options>, String> {
    let supplied = std::env::args_os()
        .skip(1)
        .map(|v| {
            v.into_string()
                .map_err(|_| "Arguments must be UTF-8".to_owned())
        })
        .collect::<Result<Vec<_>, _>>()?;
    if supplied
        .iter()
        .any(|v| matches!(v.as_str(), "--help" | "-h"))
    {
        println!("{USAGE}");
        return Ok(None);
    }
    parse_arguments(supplied).map(Some)
}

fn parse_arguments(arguments: Vec<String>) -> Result<Options, String> {
    let mut fields: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut positional = Vec::new();
    let mut arguments = arguments.into_iter();
    while let Some(argument) = arguments.next() {
        if argument.starts_with("--") {
            if ![
                "--workspace",
                "--registry",
                "--database",
                "--limits",
                "--methods",
                "--workers",
                "--supervisor",
                "--method-worker",
                "--context",
                "--claim",
                "--evidence",
                "--collection",
                "--now",
                "--goal",
                "--semantics",
                "--query-mode",
                "--preference",
                "--entry-id",
                "--label",
                "--limit",
                "--offset",
                "--path",
                "--reuse",
                "--question",
                "--source-digest",
                "--output",
            ]
            .contains(&argument.as_str())
            {
                return Err(format!("Unknown option {argument:?}"));
            }
            let value = arguments
                .next()
                .ok_or_else(|| format!("{argument} requires a value"))?;
            fields.entry(argument).or_default().push(value);
        } else if argument.starts_with('-') && argument != "-" {
            return Err(format!("Unknown option {argument:?}"));
        } else {
            positional.push(argument);
        }
    }
    let command = if positional.is_empty() {
        return Err(USAGE.into());
    } else {
        positional.remove(0)
    };
    let allowed: &[&str] = match command.as_str() {
        "describe" | "validate" | "format" | "grounded" => &[],
        "plan" => &["--claim"],
        "collect" => &["--context", "--claim", "--evidence"],
        "reason" => &["--context", "--collection", "--now"],
        "compile-aspic" => &[
            "--context",
            "--collection",
            "--goal",
            "--now",
            "--semantics",
            "--query-mode",
            "--preference",
        ],
        "explain" | "packet" => &["--claim"],
        "register" => &["--entry-id", "--label", "--claim", "--context"],
        "register-tree" => &["--context", "--limit"],
        "sources" => &["--limit", "--offset"],
        "find" => &["--claim", "--context", "--path", "--limit"],
        "assess-known" => &["--claim", "--context", "--now", "--reuse"],
        "history" => &["--source-digest", "--limit"],
        "model-context" => &["--claim", "--question", "--context", "--now", "--reuse"],
        "export-aspic" => &["--output"],
        _ => return Err(USAGE.into()),
    };
    let common = [
        "--workspace",
        "--registry",
        "--database",
        "--limits",
        "--methods",
        "--workers",
        "--supervisor",
        "--method-worker",
    ];
    for (name, values) in &fields {
        if !common.contains(&name.as_str()) && !allowed.contains(&name.as_str()) {
            return Err(format!("{name} is not accepted by {command}"));
        }
        if values.len() > 1
            && !(command == "register" && name == "--claim"
                || command == "collect" && name == "--evidence")
        {
            return Err(format!("Duplicate option {name}"));
        }
    }
    let count = positional.len();
    if match command.as_str() {
        "describe" | "sources" => count != 0,
        "find" | "register-tree" => count > 1,
        _ => count != 1,
    } {
        return Err(USAGE.into());
    }
    let value = |name: &str| fields.get(name).and_then(|v| v.first());
    for required in match command.as_str() {
        "plan" | "assess-known" => vec!["--claim"],
        "model-context" => vec!["--claim", "--question"],
        "collect" | "reason" => vec!["--context"],
        "compile-aspic" => vec!["--context", "--collection", "--goal"],
        "export-aspic" => vec!["--output"],
        _ => vec![],
    } {
        if value(required).is_none() {
            return Err(format!("{command} requires {required}"));
        }
    }
    if command == "collect" && value("--claim").is_some() && value("--evidence").is_some() {
        return Err("--claim and --evidence are mutually exclusive".into());
    }
    if value("--reuse").is_some_and(|v| !matches!(v.as_str(), "compatible" | "fresh"))
        || value("--semantics")
            .is_some_and(|v| !matches!(v.as_str(), "grounded" | "preferred" | "stable"))
        || value("--query-mode").is_some_and(|v| !matches!(v.as_str(), "credulous" | "sceptical"))
        || value("--preference").is_some_and(|v| {
            !matches!(
                v.as_str(),
                "minimum_rank" | "last_link_rank" | "last_link_partial"
            )
        })
    {
        return Err("Invalid reuse or formal policy".into());
    }
    let workspace = value("--workspace")
        .map(Into::into)
        .unwrap_or(std::env::current_dir().map_err(|e| e.to_string())?);
    let mut runtime = RuntimeOptions::new(workspace);
    runtime.registry = value("--registry").map(Into::into);
    runtime.database = value("--database").map(Into::into);
    runtime.limits = value("--limits").map(Into::into);
    runtime.methods = value("--methods")
        .cloned()
        .unwrap_or_else(|| "builtin".into());
    runtime.supervisor = value("--supervisor").map(Into::into);
    runtime.method_worker = value("--method-worker").map(Into::into);
    if let Some(workers) = value("--workers") {
        runtime.workers = workers
            .parse()
            .map_err(|_| "--workers requires an integer")?;
    }
    Ok(Options {
        runtime,
        command,
        positional: positional.pop(),
        fields,
    })
}
