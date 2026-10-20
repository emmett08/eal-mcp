//! Parse explicit development commands and reject incompatible options.
use std::path::PathBuf;

const USAGE: &str = "usage: eal-core [--workspace DIR] [--limits FILE] [--methods builtin|engineering|aspic|all] <COMMAND>\n\
    compile SOURCE                  Emit composed semantic entities\n\
    validate SOURCE                 Emit ordered static diagnostics\n\
    format SOURCE                   Emit checked canonical source\n\
    plan SOURCE --claim NAME        Plan complete evidence dependencies\n\
    evaluate SOURCE --now ISO_TIME [--context JSON|@FILE] [--records FILE]\n\
                                    Assess supplied observations without collecting\n\
    grounded GRAPH                  Solve arguments/attacks JSON\n\
    aspic THEORY                    Solve an explicit {theory: ...} JSON request\n\
    compile-aspic SOURCE --claim NAME --now ISO_TIME [--context JSON|@FILE] [--records FILE]\n\
                                    Compile and solve an observation-bound EAL snapshot\n\
      [--semantics grounded|preferred|stable] [--query-mode credulous|sceptical]\n\
      [--preference minimum_rank|last_link_rank|last_link_partial]\n\
    export-aspic RESULT              Recompute and emit checked aspic-view/3 JSON\n\
    describe                        Describe pure method and quantity contracts\n\
SOURCE may be - for standard input. Limits use a TOML [limits] table.\n\
All file inputs and trusted imports stay within the selected workspace.";

pub(crate) struct Options {
    pub(crate) workspace: PathBuf,
    pub(crate) limits: Option<String>,
    pub(crate) command: String,
    pub(crate) source: Option<String>,
    pub(crate) context: Option<String>,
    pub(crate) records: Option<String>,
    pub(crate) now: Option<String>,
    pub(crate) claim: Option<String>,
    pub(crate) methods: String,
    pub(crate) semantics: Option<String>,
    pub(crate) query_mode: Option<String>,
    pub(crate) preference: Option<String>,
}

pub(crate) fn parse() -> Result<Option<Options>, String> {
    let mut supplied = Options {
        workspace: std::env::current_dir().map_err(|e| e.to_string())?,
        limits: None,
        command: String::new(),
        source: None,
        context: None,
        records: None,
        now: None,
        claim: None,
        methods: "builtin".into(),
        semantics: None,
        query_mode: None,
        preference: None,
    };
    let arguments = std::env::args_os()
        .skip(1)
        .map(|value| {
            value
                .into_string()
                .map_err(|_| "Arguments must be UTF-8".to_owned())
        })
        .collect::<Result<Vec<_>, _>>()?;
    let mut arguments = arguments.into_iter();
    while let Some(argument) = arguments.next() {
        if matches!(argument.as_str(), "--help" | "-h") {
            println!("{USAGE}");
            return Ok(None);
        }
        match argument.as_str() {
            "--workspace" | "--limits" | "--context" | "--records" | "--now" | "--claim"
            | "--methods" | "--semantics" | "--query-mode" | "--preference" => {
                let value = arguments
                    .next()
                    .ok_or_else(|| format!("{argument} requires a value"))?;
                match argument.as_str() {
                    "--workspace" => supplied.workspace = value.into(),
                    "--limits" => supplied.limits = Some(value),
                    "--context" => supplied.context = Some(value),
                    "--records" => supplied.records = Some(value),
                    "--now" => supplied.now = Some(value),
                    "--claim" => supplied.claim = Some(value),
                    "--methods" => supplied.methods = value,
                    "--semantics" => supplied.semantics = Some(value),
                    "--query-mode" => supplied.query_mode = Some(value),
                    "--preference" => supplied.preference = Some(value),
                    _ => unreachable!(),
                }
            }
            value if value.starts_with('-') && value != "-" => {
                return Err(format!("Unknown option {value:?}\n{USAGE}"));
            }
            _ if supplied.command.is_empty() => supplied.command = argument,
            _ if supplied.source.is_none() => supplied.source = Some(argument),
            _ => return Err(USAGE.into()),
        }
    }
    if !matches!(
        supplied.command.as_str(),
        "compile"
            | "validate"
            | "format"
            | "plan"
            | "evaluate"
            | "grounded"
            | "describe"
            | "aspic"
            | "compile-aspic"
            | "export-aspic"
    ) || !matches!(
        supplied.methods.as_str(),
        "builtin" | "engineering" | "aspic" | "all"
    ) || (supplied.command != "describe") != supplied.source.is_some()
        || (matches!(supplied.command.as_str(), "evaluate" | "compile-aspic")
            && supplied.now.is_none())
        || matches!(supplied.command.as_str(), "plan" | "compile-aspic") != supplied.claim.is_some()
        || (!matches!(supplied.command.as_str(), "evaluate" | "compile-aspic")
            && (supplied.now.is_some() || supplied.context.is_some() || supplied.records.is_some()))
        || (supplied.command != "compile-aspic"
            && (supplied.semantics.is_some()
                || supplied.query_mode.is_some()
                || supplied.preference.is_some()))
        || supplied
            .semantics
            .as_deref()
            .is_some_and(|v| !matches!(v, "grounded" | "preferred" | "stable"))
        || supplied
            .query_mode
            .as_deref()
            .is_some_and(|v| !matches!(v, "credulous" | "sceptical"))
        || supplied
            .preference
            .as_deref()
            .is_some_and(|v| !matches!(v, "minimum_rank" | "last_link_rank" | "last_link_partial"))
    {
        return Err(USAGE.into());
    }
    Ok(Some(supplied))
}
