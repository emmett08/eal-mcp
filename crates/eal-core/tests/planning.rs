//! Check complete evidence closure, scope and acquisition ordering for authored support and objection routes.
use eal_core::{ExecutionLimits, composition, planning::EvidencePlanner};
const SOURCE: &str = include_str!("support/planning.eal");

#[test]
fn all_alternatives_and_defence_sources_are_included_once() {
    let program = composition::compile(SOURCE, None, "", ExecutionLimits::default()).unwrap();
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
    let mut program = composition::compile(SOURCE, None, "", ExecutionLimits::default()).unwrap();
    program.tools.clear();
    assert!(EvidencePlanner::new(&program).is_err());
    program.limits.declarations = 0;
    assert!(EvidencePlanner::new(&program).is_err());
}
