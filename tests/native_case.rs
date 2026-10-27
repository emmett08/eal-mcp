//! Authored synthetic observations for current native API and command tests.
#![allow(dead_code)]
use eal_core::{ExecutionLimits, canonical, composition, evaluation};
use serde_json::{Value, json};

pub const NOW: &str = "2026-10-24T12:00:00Z";
pub const SOURCE: &str = r#"language "EAL/3"
environment lab {
 require site == "bench"
}
tool reader {
 version "1"
}
evidence observed {
 tool reader
 kind test
 environment lab
 max_age 60
 input {"sample":1}
 require passed == true
}
reasoning measured {
 method "structured/1"
 rationale "The measured test supports its declared result."
}
claim working {
 statement "The measured test passed at this bench."
 environment lab
}
claim downstream {
 statement "The downstream claim follows from the measured test."
 environment lab
}
argument support = [evidence observed] via measured => working
argument propagate = [premises working] via measured => downstream
"#;

pub fn observation(source: &str, name: &str, context: &Value, value: Value) -> Value {
    let program = composition::compile(source, None, "", ExecutionLimits::default()).unwrap();
    let evidence = &program.evidence[name];
    let tool = &program.tools[evidence.tool.as_ref().unwrap()];
    let acquisition = json!({"tool":tool.name,"tool_version":tool.version,"input":evidence.input,"context":context});
    let full = json!({"evidence_id":name,"environment":evidence.environment,"tool":tool.name,"tool_version":tool.version,"input":evidence.input,"context":context});
    json!({"schema":"EAL/observation-record/1","evidence_id":name,"source_digest":program.source_digest,
        "tool":tool.name,"tool_version":tool.version,"tool_binding_digest":"0".repeat(64),
        "evidence_kind":evidence.kind,"environment":evidence.environment,
        "environment_fingerprint":evaluation::environment_fingerprint(&evidence.environment,context,&program.limits).unwrap(),
        "input_digest":canonical::canonical_digest(&evidence.input,&program.limits).unwrap(),
        "input":evidence.input,"context":context,"acquisition_request":acquisition,
        "acquisition_request_digest":canonical::canonical_digest(&acquisition,&program.limits).unwrap(),
        "request_digest":canonical::canonical_digest(&full,&program.limits).unwrap(),
        "collected_at":NOW,"run_id":format!("native-synthetic-{name}"),"status":"ok",
        "data_digest":canonical::canonical_digest(&value,&program.limits).unwrap(),"value":value})
}
pub fn case() -> Value {
    let context = json!({"site":"bench"});
    let record = observation(SOURCE, "observed", &context, json!({"passed":true}));
    json!({"source":SOURCE,"records":{"observed":record},"context":context,"now":NOW,"goal":"working","claim":"working"})
}
pub fn theory() -> Value {
    json!({"theory":{"premises":[{"atom":"checked","kind":"axiom"}],
        "rules":[{"id":"accept","kind":"defeasible","antecedents":["checked"],"consequent":"accepted","name":"applicable","rank":500}],
        "contraries":[],"goal":"accepted"}})
}
pub fn typed_case() -> Value {
    let payload = theory();
    let source = format!(
        r#"language "EAL/3"
environment lab {{
 require site == "bench"
}}
tool reader {{
 version "1"
}}
evidence formal {{
 tool reader
 kind aspic_theory
 environment lab
 max_age 60
 require schema == "EAL/typed-input/1"
}}
reasoning solve {{
 method "argumentation/aspic/2"
 rationale "The bounded theory determines the selected proposition."
}}
claim accepted {{
 statement "The declared theory accepts the selected proposition."
 environment lab
 proposition {{
  subject "native-acceptance"
  quantity "proposition"
  unit "1"
  scope "synthetic-case"
  valid_from "2026-10-24T11:00:00Z"
  valid_until "2026-10-24T13:00:00Z"
  query {payload}
  result "query_accepted" == true
 }}
}}
argument formal_arg = [evidence formal] via solve => accepted binding formal
"#
    );
    let context = json!({"site":"bench"});
    let value = json!({"schema":"EAL/typed-input/1","method":"argumentation/aspic/2",
        "subject":"native-acceptance","quantity":"proposition","unit":"1","scope":"synthetic-case",
        "valid_from":"2026-10-24T11:00:00Z","valid_until":"2026-10-24T13:00:00Z","payload":payload});
    let record = observation(&source, "formal", &context, value);
    json!({"source":source,"records":{"formal":record},"context":context,"now":NOW,"goal":"accepted"})
}
