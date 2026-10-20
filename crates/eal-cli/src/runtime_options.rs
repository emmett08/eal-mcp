//! Explicit operator options for the staged runtime development command.
use std::{collections::BTreeSet, path::PathBuf};

pub const USAGE: &str = "usage: eal-runtime [--workspace DIR] [--registry TOML] [--database SQLITE]\n\
    [--limits TOML] [--methods builtin|engineering|aspic|all] [--workers 1..32]\n\
    [--supervisor EXECUTABLE] [--method-worker EXECUTABLE] COMMAND\n\
validate SOURCE\nformat SOURCE\nplan SOURCE --claim NAME\n\
collect SOURCE --context JSON|@FILE [--claim NAME | --evidence NAME ...]\n\
reason SOURCE --context JSON|@FILE [--collection ID] [--now ISO_TIME]\n\
compile-aspic SOURCE --context JSON|@FILE --collection ID --goal NAME\n\
    [--now ISO_TIME] [--semantics grounded|preferred|stable]\n\
    [--query-mode credulous|sceptical]\n\
    [--preference minimum_rank|last_link_rank|last_link_partial]\n\
explain ASSESSMENT_ID [--claim NAME]\n\
Companion executables default to the directory containing eal-runtime.\n\
Commands use durable private POSIX storage. This staged driver has no MCP transport.";

pub struct Options {
    pub workspace: PathBuf,
    pub registry: Option<PathBuf>,
    pub database: Option<PathBuf>,
    pub limits: Option<String>,
    pub methods: String,
    pub workers: usize,
    pub supervisor: Option<PathBuf>,
    pub method_worker: Option<PathBuf>,
    pub command: String,
    pub source: String,
    pub context: Option<String>,
    pub collection: Option<String>,
    pub claim: Option<String>,
    pub goal: Option<String>,
    pub evidence: Vec<String>,
    pub now: Option<String>,
    pub semantics: String,
    pub query_mode: String,
    pub preference: Option<String>,
}

pub fn parse() -> Result<Option<Options>, String> {
    let mut options = Options {
        workspace: std::env::current_dir().map_err(|e| e.to_string())?,
        registry: None,
        database: None,
        limits: None,
        methods: "builtin".into(),
        workers: 4,
        supervisor: None,
        method_worker: None,
        command: String::new(),
        source: String::new(),
        context: None,
        collection: None,
        claim: None,
        goal: None,
        evidence: Vec::new(),
        now: None,
        semantics: "grounded".into(),
        query_mode: "sceptical".into(),
        preference: None,
    };
    let mut seen = BTreeSet::new();
    let mut arguments = std::env::args_os()
        .skip(1)
        .map(|v| {
            v.into_string()
                .map_err(|_| "Arguments must be UTF-8".to_owned())
        })
        .collect::<Result<Vec<_>, _>>()?
        .into_iter();
    while let Some(argument) = arguments.next() {
        if matches!(argument.as_str(), "--help" | "-h") {
            println!("{USAGE}");
            return Ok(None);
        }
        if argument.starts_with("--") {
            if !matches!(
                argument.as_str(),
                "--workspace"
                    | "--registry"
                    | "--database"
                    | "--limits"
                    | "--methods"
                    | "--workers"
                    | "--supervisor"
                    | "--method-worker"
                    | "--context"
                    | "--collection"
                    | "--claim"
                    | "--goal"
                    | "--evidence"
                    | "--now"
                    | "--semantics"
                    | "--query-mode"
                    | "--preference"
            ) {
                return Err(format!("Unknown option {argument:?}"));
            }
            if argument != "--evidence" && !seen.insert(argument.clone()) {
                return Err(format!("Duplicate option {argument}"));
            }
            let value = arguments
                .next()
                .ok_or_else(|| format!("{argument} requires a value"))?;
            match argument.as_str() {
                "--workspace" => options.workspace = value.into(),
                "--registry" => options.registry = Some(value.into()),
                "--database" => options.database = Some(value.into()),
                "--limits" => options.limits = Some(value),
                "--methods" => options.methods = value,
                "--workers" => {
                    options.workers = value.parse().map_err(|_| "--workers requires an integer")?
                }
                "--supervisor" => options.supervisor = Some(value.into()),
                "--method-worker" => options.method_worker = Some(value.into()),
                "--context" => options.context = Some(value),
                "--collection" => options.collection = Some(value),
                "--claim" => options.claim = Some(value),
                "--goal" => options.goal = Some(value),
                "--evidence" => options.evidence.push(value),
                "--now" => options.now = Some(value),
                "--semantics" => options.semantics = value,
                "--query-mode" => options.query_mode = value,
                "--preference" => options.preference = Some(value),
                _ => unreachable!(),
            }
        } else if argument.starts_with('-') && argument != "-" {
            return Err(format!("Unknown option {argument:?}"));
        } else if options.command.is_empty() {
            options.command = argument;
        } else if options.source.is_empty() {
            options.source = argument;
        } else {
            return Err(USAGE.into());
        }
    }
    let collects = options.command == "collect";
    let reasons = options.command == "reason";
    let formal = options.command == "compile-aspic";
    if !matches!(
        options.command.as_str(),
        "validate" | "format" | "plan" | "collect" | "reason" | "compile-aspic" | "explain"
    ) || options.source.is_empty()
        || !matches!(
            options.methods.as_str(),
            "builtin" | "engineering" | "aspic" | "all"
        )
        || (collects || reasons || formal) != options.context.is_some()
        || (options.collection.is_some() && !reasons && !formal)
        || (formal && options.collection.is_none())
        || (options.claim.is_some()
            && !matches!(options.command.as_str(), "plan" | "collect" | "explain"))
        || (options.command == "plan" && options.claim.is_none())
        || (formal != options.goal.is_some())
        || (!options.evidence.is_empty() && (!collects || options.claim.is_some()))
        || (options.now.is_some() && !reasons && !formal)
        || (!formal
            && ["--semantics", "--query-mode", "--preference"]
                .iter()
                .any(|option| seen.contains(*option)))
        || !matches!(
            options.semantics.as_str(),
            "grounded" | "preferred" | "stable"
        )
        || !matches!(options.query_mode.as_str(), "credulous" | "sceptical")
        || options
            .preference
            .as_deref()
            .is_some_and(|p| !matches!(p, "minimum_rank" | "last_link_rank" | "last_link_partial"))
        || options.evidence.iter().collect::<BTreeSet<_>>().len() != options.evidence.len()
    {
        return Err(USAGE.into());
    }
    Ok(Some(options))
}
