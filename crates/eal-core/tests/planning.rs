//! Check complete evidence closure, scope and acquisition ordering against retained source plans.
use eal_core::{ExecutionLimits, composition, planning::EvidencePlanner};
use serde_json::{Value, json};

#[test]
fn planning_matches_every_source_order_and_closure_fixture() {
    let fixture: Value =
        serde_json::from_str(include_str!("../../../fixtures/planning/reference.json")).unwrap();
    for case in fixture["cases"].as_array().unwrap() {
        let program = composition::compile(
            case["source"].as_str().unwrap(),
            None,
            "",
            ExecutionLimits::default(),
        )
        .unwrap();
        let planner = EvidencePlanner::new(&program).unwrap();
        let planned = planner.plan(case["claim"].as_str().unwrap());
        if let Some(error) = case["error"].as_str() {
            assert_eq!(planned.unwrap_err(), error, "{}", case["name"]);
        } else {
            let plan = planned.unwrap();
            assert_eq!(
                serde_json::to_value(&plan).unwrap(),
                case["result"],
                "{}",
                case["name"]
            );
            assert_eq!(json!(plan.evidence_ids()), case["evidence_ids"]);
            assert_eq!(
                plan.estimated_calls(),
                case["estimated_calls"].as_u64().unwrap() as usize
            );
            assert_eq!(plan, planner.plan(&plan.claim).unwrap());
        }
    }
}

#[test]
fn all_alternatives_and_defence_sources_are_included_once() {
    let fixture: Value =
        serde_json::from_str(include_str!("../../../fixtures/planning/reference.json")).unwrap();
    let case = &fixture["cases"][0];
    let program = composition::compile(
        case["source"].as_str().unwrap(),
        None,
        "",
        ExecutionLimits::default(),
    )
    .unwrap();
    let plan = EvidencePlanner::new(&program)
        .unwrap()
        .plan("root")
        .unwrap();
    assert_eq!(
        plan.evidence_ids(),
        [
            "alternative",
            "direct",
            "backing_record",
            "validation",
            "premise",
            "challenge",
            "defence",
            "reply",
            "objection_premise"
        ]
    );
    assert_eq!(
        plan.closure.claims.into_iter().collect::<Vec<_>>(),
        ["attacker", "leaf", "root"]
    );
    assert!(!plan.closure.objections.contains("scoped_out"));
}

#[test]
fn invalid_programmes_are_rejected_before_reference_indexing() {
    let fixture: Value =
        serde_json::from_str(include_str!("../../../fixtures/planning/reference.json")).unwrap();
    let case = &fixture["cases"][0];
    let mut program = composition::compile(
        case["source"].as_str().unwrap(),
        None,
        "",
        ExecutionLimits::default(),
    )
    .unwrap();
    program.tools.clear();
    assert!(EvidencePlanner::new(&program).is_err());
    program.limits.declarations = 0;
    assert!(EvidencePlanner::new(&program).is_err());
}
