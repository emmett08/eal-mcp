//! Orchestrate native export acceptance against the exact downstream validator.
use eal_runtime::process::Supervisor;
use input::{Options, Temporary, checked_consumer, executable, load, parse_json, read_bytes};
use policy::check_schema_and_policy;
use process::Runner;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};
use views::{generated_views, put};
const CONSUMER_REVISION: &str = "1c5523193669be98aeaa6290e54db7d91552c4de";
const CONSUMER_BLOB: &str = "16512f96824d5b218e5b790b79777a6f16217d21";
const MAX_BYTES: usize = 4 * 1024 * 1024;
const MAX_INPUT_BYTES: usize = 64 * 1024 * 1024;
const MAX_ARGUMENTS: u64 = 4096;
const MAX_DEFEATS: u64 = 65_536;
type Result<T> = std::result::Result<T, String>;
mod corpus;
mod input;
mod policy;
mod process;
mod views;

const GRAPH_RUNNER: &str = r#"
import {readFileSync} from 'node:fs';
import {validateGraph} from './graph.ts';
const cases=JSON.parse(readFileSync(process.argv[2],'utf8'));
for(const item of cases){
  try{validateGraph(item.view);}
  catch(error){throw new Error(`${item.name}: ${error.message}`);}
}
const base=cases.find(item=>item.view.defeats.length>0 &&
  item.view.arguments.some(a=>a.top==='defeasible')).view;
const rejected=[];
for(const [name,change] of [
  ['duplicate IDs',v=>v.arguments.push(structuredClone(v.arguments[0]))],
  ['wrong strength',v=>v.arguments[0].strength=0],
  ['wrong grounded label',v=>v.arguments[0].label=v.arguments[0].label==='in'?'out':'in'],
  ['duplicate witness',v=>v.defeats.push(structuredClone(v.defeats[0]))],
  ['wrong query status',v=>v.query_status=v.query_status==='accepted'?'rejected':'accepted'],
  ['unknown extension member',v=>v.extensions[0].push('unknown')],
  ['derivation cycle',v=>{const a=v.arguments.find(a=>a.top==='defeasible');a.direct_subarguments=[a.id];}]
]){
  const view=structuredClone(base);change(view);
  try{validateGraph(view);}catch{rejected.push(name);}
}
if(rejected.length!==7)throw new Error('A forged-graph control was not rejected');
console.log(JSON.stringify({graph_views:cases.length,rejected_controls:rejected}));
"#;

/// Check current native exports against the selected downstream consumer.
pub fn run(args: &[String]) -> Result<Value> {
    let options = Options::parse(args)?;
    // File admission precedes executable discovery and process side effects.
    let cached_views = options.views.as_ref().map(|path| load(path)).transpose()?;
    let graph = checked_consumer(&options.consumer)?;
    let root = options
        .root
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."));
    let binary = executable(&options.binary)?;
    let supervisor = executable(&options.supervisor.unwrap_or_else(|| {
        binary
            .parent()
            .expect("absolute executable")
            .join("eal-process-supervisor")
    }))?;
    let node = executable(options.node.as_deref().unwrap_or(Path::new("node")))?;
    let temporary = Temporary::new()?;
    let mut environment = BTreeMap::from([("LANG".into(), "C.UTF-8".into())]);
    environment.insert("PATH".into(), String::new());
    let runner = Runner {
        supervisor: Supervisor::new(supervisor),
        environment,
    };
    let version =
        String::from_utf8(runner.run(&node, &["--version".into()], &temporary.0, 10, 4096)?)
            .map_err(|e| e.to_string())?;
    let version = version.trim();
    if version
        .trim_start_matches('v')
        .split('.')
        .next()
        .and_then(|s| s.parse::<u64>().ok())
        .is_none_or(|major| major < 24)
    {
        return Err(format!("Node >=24 is required, found {version}"));
    }
    let (explicit, compiled) = corpus::cases();
    let views = match cached_views {
        Some(value) => value
            .as_array()
            .ok_or("Cached views must be an array")?
            .clone(),
        None => generated_views(
            &runner,
            &binary,
            &temporary.0.join("work"),
            &explicit,
            &compiled,
        )?,
    };
    check_schema_and_policy(&root, &views, &explicit, &compiled)?;
    put(&temporary.0, "views.json", &json!(views))?;
    put(&temporary.0, "graph.ts", &Value::String(graph))?;
    put(
        &temporary.0,
        "check.mjs",
        &Value::String(GRAPH_RUNNER.into()),
    )?;
    let output = runner.run(
        &node,
        &[
            temporary.0.join("check.mjs").to_string_lossy().into_owned(),
            temporary
                .0
                .join("views.json")
                .to_string_lossy()
                .into_owned(),
        ],
        &temporary.0,
        60,
        MAX_BYTES,
    )?;
    let mut summary = parse_json(&output, MAX_BYTES)?;
    if summary["graph_views"].as_u64() != Some(views.len() as u64)
        || summary["rejected_controls"].as_array().map(Vec::len) != Some(7)
    {
        return Err("Consumer returned an incomplete acceptance summary".into());
    }
    if let Some(path) = options.save_views {
        super::write_json(&path, &json!(views))?;
    }
    let object = summary
        .as_object_mut()
        .ok_or("Consumer summary must be an object")?;
    object.extend(json!({
        "consumer_revision":CONSUMER_REVISION,"consumer_git_blob":CONSUMER_BLOB,
        "explicit_views":explicit.len(),"compiled_views":compiled.len(),
        "published_schema":"native JSON Schema 2020-12: passed","consumer_policy":"passed",
        "node":version,"provided_binary_sha256":format!("{:x}",Sha256::digest(read_bytes(&binary,1024*1024*1024)?)),
        "views":if options.views.is_some(){"cached; native commands not rerun"}else{"fresh native CLI output; PATH empty"},
        "scope":"Unmodified consumer graph function bodies plus separately validated schema/policy; no Ajv, Vue, browser or live bridge",
        "orchestrator":"eal-smoke consumer (Rust)"
    }).as_object().expect("literal object").clone());
    Ok(summary)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn git_blob_matches_published_git_empty_blob() {
        assert_eq!(
            input::git_blob(b""),
            "e69de29bb2d1d6434b8b29ae775ad8c2e48c5391"
        );
    }
    #[test]
    fn invalid_consumer_and_duplicate_json_fail_closed() {
        let temp = Temporary::new().unwrap();
        let path = temp.0.join("consumer.ts");
        std::fs::write(&path, b"export function validateGraph() {}").unwrap();
        assert!(
            checked_consumer(&path)
                .unwrap_err()
                .contains("Expected pinned consumer blob")
        );
        assert!(parse_json(br#"{"x":1,"x":2}"#, 1024).is_err());
    }
    #[test]
    fn imported_fixture_paths_stay_in_owned_workspace() {
        let temp = Temporary::new().unwrap();
        for name in ["../outside", "/outside", "nested/../../outside", ""] {
            assert!(put(&temp.0, name, &json!("x")).is_err());
        }
        put(&temp.0, "nested/import.eal", &json!("language \"EAL/4\"\n")).unwrap();
    }
    #[test]
    fn options_preserve_both_explicit_and_positional_interfaces() {
        for args in [
            vec!["consumer.ts", "eal-core"],
            vec!["--consumer", "consumer.ts", "--binary", "eal-core"],
        ] {
            let args = args.into_iter().map(str::to_owned).collect::<Vec<_>>();
            let options = Options::parse(&args).unwrap();
            assert_eq!(options.consumer, Path::new("consumer.ts"));
        }
        assert!(
            Options::parse(&[
                "--consumer".into(),
                "a".into(),
                "--consumer".into(),
                "b".into()
            ])
            .is_err()
        );
    }
    #[test]
    fn cached_views_require_exact_fixture_names() {
        let explicit = vec![json!({"name":"case"})];
        let error = check_schema_and_policy(
            Path::new("unused"),
            &[
                json!({"name":"explicit-case"}),
                json!({"name":"explicit-case"}),
            ],
            &explicit,
            &[],
        )
        .unwrap_err();
        assert!(error.contains("exactly"));
    }
}
