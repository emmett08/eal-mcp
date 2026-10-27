//! Authored native scenarios exercise the current downstream graph contract.
#[path = "../../../../../tests/native_case.rs"]
mod native_case;
use serde_json::{Value, json};

fn typed_missing() -> Value {
    let source = r#"language "EAL/4"
environment lab {
 require site == "bench"
}
tool reader {
 version "1"
}
evidence trial {
 tool reader
 kind experiment
 environment lab
 max_age 60
 require schema == "EAL/typed-input/1"
}
reasoning contrast {
 method "causal/1"
 rationale "The selected experiment estimates the pressure difference."
}
claim raised {
 statement "The measured pressure difference exceeds five kilopascals."
 environment lab
 proposition {
  subject "synthetic-pump"
  quantity "pressure"
  unit "kPa"
  scope "synthetic-case"
  valid_from "2026-10-24T11:00:00Z"
  valid_until "2026-10-24T13:00:00Z"
  query {"assignment":"randomised"}
  result "estimate" >= 5
 }
}
argument measured = [evidence trial] via contrast => raised binding trial
"#;
    json!({"source":source,"records":{},"context":{"site":"bench"},"now":native_case::NOW,"goal":"raised"})
}

pub(super) fn cases() -> (Vec<Value>, Vec<Value>) {
    let theories = [
        (
            "undercut",
            json!({"premises":[{"atom":"p","kind":"ordinary","rank":500},{"atom":"q","kind":"ordinary","rank":500}],
            "rules":[{"id":"derive","kind":"defeasible","antecedents":["p"],"consequent":"x","name":"use_x","rank":500},
                {"id":"cancel","kind":"strict","antecedents":["q"],"consequent":"~use_x"}],
            "contraries":[{"attacker":"~use_x","target":"use_x"}],"goal":"x"}),
        ),
        (
            "mutual",
            json!({"premises":[{"atom":"p","kind":"ordinary","rank":500},{"atom":"q","kind":"ordinary","rank":500}],
            "rules":[],"contraries":[{"attacker":"p","target":"q"},{"attacker":"q","target":"p"}],"goal":"p"}),
        ),
        (
            "odd_cycle",
            json!({"premises":[{"atom":"p","kind":"ordinary","rank":500},{"atom":"q","kind":"ordinary","rank":500},{"atom":"r","kind":"ordinary","rank":500}],
            "rules":[],"contraries":[{"attacker":"p","target":"q"},{"attacker":"q","target":"r"},{"attacker":"r","target":"p"}],"goal":"p"}),
        ),
        ("accepted", native_case::theory()["theory"].clone()),
    ];
    let mut explicit = Vec::new();
    let mut compiled = Vec::new();
    for semantics in ["grounded", "preferred", "stable"] {
        for query_mode in ["credulous", "sceptical"] {
            for preference in ["minimum_rank", "last_link_rank", "last_link_partial"] {
                for (name, theory) in &theories {
                    let mut theory = theory.clone();
                    theory["options"] = json!({"semantics":semantics,"query_mode":query_mode,"preference":preference});
                    explicit.push(json!({"name":format!("{name}-{semantics}-{query_mode}-{preference}"),"payload":{"theory":theory}}));
                }
                for (name, mut case) in [
                    ("measured", native_case::case()),
                    ("missing", native_case::case()),
                    ("typed_missing", typed_missing()),
                ] {
                    if name == "missing" {
                        case["records"] = json!({});
                    }
                    case["name"] = json!(format!("{name}-{semantics}-{query_mode}-{preference}"));
                    case["semantics"] = json!(semantics);
                    case["query_mode"] = json!(query_mode);
                    case["preference"] = json!(preference);
                    compiled.push(case);
                }
            }
        }
    }
    (explicit, compiled)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_scenarios_cover_all_semantics_queries_and_preferences() {
        let (explicit, compiled) = cases();
        assert_eq!(explicit.len(), 72);
        assert_eq!(compiled.len(), 54);
        assert!(
            explicit
                .iter()
                .any(|c| c["payload"]["theory"]["options"]["semantics"] == "stable")
        );
        let registry = eal_aspic::aspic_registry();
        let typed = native_case::typed_case();
        let program = eal_core::composition::compile(
            typed["source"].as_str().unwrap(),
            None,
            "",
            eal_core::ExecutionLimits::default(),
        )
        .unwrap();
        assert!(eal_core::validation::validate(&program, &registry).is_empty());
        let result = eal_core::evaluation::evaluate(
            &program,
            &typed["records"],
            typed["now"].as_str().unwrap(),
            &typed["context"],
            &registry,
            None,
        )
        .unwrap();
        assert_eq!(
            result["claims"]["accepted"]["status"], "supported",
            "{result}"
        );
    }
}
