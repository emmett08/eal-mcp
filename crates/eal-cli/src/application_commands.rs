//! Dispatch CLI requests to pure solvers or the durable application façade.
use crate::application_options::Options;
use eal_cli::input::Workspace;
use eal_runtime::{
    bootstrap,
    catalogue::{FindOptions, RegisterOptions},
    knowledge::{EALKnowledgeBase, ModelContextAdapter},
    registered_assessment::{RegisteredAssessmentOptions, ReusePolicy},
    service::FormalOptions,
};
use serde_json::{Value, json};
use std::sync::Arc;

pub fn run(options: Options) -> Result<Value, String> {
    let workspace = Workspace::new(&options.runtime.workspace)?;
    let limits = bootstrap::load_limits(options.runtime.limits.as_deref())?;
    let argument = options.positional.as_deref().unwrap_or(".");
    if matches!(options.command.as_str(), "grounded" | "export-aspic") {
        let payload = workspace.json_file(argument, &limits)?;
        if options.command == "grounded" {
            let object = payload
                .as_object()
                .filter(|o| {
                    o.len() == 2 && o.contains_key("arguments") && o.contains_key("attacks")
                })
                .ok_or("Graph must contain exactly arguments and attacks")?;
            return eal_core::dialectic::solve_grounded(
                &object["arguments"],
                &object["attacks"],
                &limits,
            );
        }
        let view = eal_aspic::export_aspic_view(&payload, &limits)?;
        let encoded = format!(
            "{}\n",
            serde_json::to_string_pretty(&view).map_err(|e| e.to_string())?
        );
        let output = workspace.write_output(options.required("--output")?, encoded.as_bytes())?;
        return Ok(
            json!({"output":output,"argument_count":view["arguments"].as_array().map(Vec::len),"defeat_witness_count":view["defeats"].as_array().map(Vec::len),"formal_status":view["formal_status"]}),
        );
    }
    let service = Arc::new(bootstrap::build_service(options.runtime.clone())?);
    let context = options
        .value("--context")
        .map(|value| workspace.context(Some(value), &limits))
        .transpose()?;
    if matches!(
        options.command.as_str(),
        "register"
            | "register-tree"
            | "sources"
            | "find"
            | "assess-known"
            | "history"
            | "model-context"
    ) {
        let knowledge = Arc::new(EALKnowledgeBase::new(service)?);
        return match options.command.as_str() {
            "register" => knowledge.register(
                argument,
                RegisterOptions {
                    entry_id: options.value("--entry-id"),
                    label: options.value("--label"),
                    claims: options.values("--claim"),
                    context: context.as_ref(),
                },
            ),
            "register-tree" => {
                knowledge.register_tree(argument, context.as_ref(), options.number("--limit", 512)?)
            }
            "sources" => Ok(
                json!({"entries":knowledge.sources(options.number("--limit",50)?,options.number("--offset",0)?)?}),
            ),
            "find" => Ok(
                json!({"matches":knowledge.find(FindOptions {query:options.positional.as_deref(),claim:options.value("--claim"),context:context.as_ref(),path:options.value("--path"),limit:options.number("--limit",50)?})?}),
            ),
            "history" => knowledge.history(
                argument,
                options.value("--source-digest"),
                options.number("--limit", 50)?,
            ),
            "assess-known" | "model-context" => {
                let selected = RegisteredAssessmentOptions {
                    context: context.as_ref(),
                    now: options.value("--now"),
                    reuse: if options.value("--reuse") == Some("fresh") {
                        ReusePolicy::Fresh
                    } else {
                        ReusePolicy::Compatible
                    },
                    ..Default::default()
                };
                if options.command == "assess-known" {
                    knowledge.assess(argument, options.required("--claim")?, selected)
                } else {
                    ModelContextAdapter::new(knowledge).prepare(
                        options.required("--question")?,
                        argument,
                        options.required("--claim")?,
                        selected,
                    )
                }
            }
            _ => unreachable!(),
        };
    }
    match options.command.as_str() {
        "describe" => service.describe(),
        "explain" => service.explain(argument, options.value("--claim")),
        "packet" => service.packet(argument, options.value("--claim")),
        _ => {
            let source = workspace.source(argument, limits.source_bytes)?;
            match options.command.as_str() {
                "validate" => Ok(service.validate(&source)),
                "format" => service.format(&source),
                "plan" => service.plan(&source, options.required("--claim")?),
                "collect" => {
                    if let Some(claim) = options.value("--claim") {
                        service.collect_claim(&source, context.as_ref().unwrap(), claim)
                    } else {
                        service.collect(
                            &source,
                            context.as_ref().unwrap(),
                            options.values("--evidence"),
                        )
                    }
                }
                "reason" => service.reason(
                    &source,
                    context.as_ref().unwrap(),
                    options.value("--collection"),
                    options.value("--now"),
                ),
                "compile-aspic" => service.compile_aspic(
                    &source,
                    context.as_ref().unwrap(),
                    options.required("--collection")?,
                    FormalOptions {
                        goal: options.required("--goal")?,
                        now: options.value("--now"),
                        semantics: options.value("--semantics").unwrap_or("grounded"),
                        query_mode: options.value("--query-mode").unwrap_or("sceptical"),
                        preference: options.value("--preference"),
                    },
                ),
                _ => unreachable!(),
            }
        }
    }
}
