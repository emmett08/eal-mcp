//! Opaque JSON literals cross text, derived structs and tagged criterion boundaries unchanged.
use eal_collector_contracts::{execution::*, strict_json_exact};
use serde_json::{Value, json};

#[test]
fn method_input_admission_has_a_closed_distinct_wire_contract() {
    let value = json!({"kind":"method_input","method":"counterfactual/1"});
    let decoded: EvaluationCriterion = serde_json::from_value(value.clone()).unwrap();
    assert_eq!(serde_json::to_value(decoded).unwrap(), value);
    for invalid in [
        json!({"kind":"method_input"}),
        json!({"kind":"method_input","method":12}),
        json!({"kind":"method_input","method":"counterfactual/1","supported":true}),
    ] {
        assert!(serde_json::from_value::<EvaluationCriterion>(invalid).is_err());
    }
}

#[test]
fn exact_literal_arguments_scope_and_acceptance_survive_text_and_value_decoding() {
    let literal=strict_json_exact(br#"{"$serde_json::private::Number":"literal","$serde_json::private::RawValue":"literal","n":90071992547409931234567891,"decimal":0.12345678901234567890123456789}"#,Default::default()).unwrap();
    let criterion = json!({"kind":"equals","expected":literal});
    let text = serde_json::to_vec(&criterion).unwrap();
    for parsed in [
        serde_json::from_slice::<EvaluationCriterion>(&text),
        serde_json::from_value::<EvaluationCriterion>(criterion),
    ] {
        match parsed.unwrap() {
            EvaluationCriterion::Equals { expected } => assert_eq!(expected, literal),
            _ => panic!("wrong criterion"),
        }
    }
    let request = json!({"evidence_id":"e","environment":"dev","tool":"test","tool_version":"1","input":literal,"context":{"nested":literal}});
    let decoded: eal_collector_contracts::AcquisitionRequest =
        serde_json::from_value(request.clone()).unwrap();
    assert_eq!(decoded.input, literal);
    assert_eq!(decoded.context, json!({"nested":literal}));
    let amount: Value = serde_json::from_str("90071992547409931234567891").unwrap();
    let criterion = json!({"kind":"numeric","comparison":"equal","threshold":amount,"value_unit":"1","threshold_unit":"1"});
    let parsed: EvaluationCriterion = serde_json::from_value(criterion).unwrap();
    match parsed {
        EvaluationCriterion::Numeric { threshold, .. } => assert_eq!(threshold, amount),
        _ => panic!("wrong criterion"),
    }
}

#[test]
fn scoped_retained_judgement_wire_preserves_exact_scope_and_legacy_missing_scope() {
    use eal_collector_contracts::capability::SubjectScope;
    use std::collections::BTreeMap;
    let scope = SubjectScope {
        kind: "design".into(),
        subject: "controller revision abc123".into(),
        selectors: BTreeMap::from([("revision".into(), json!("abc123"))]),
        identity_dimensions: vec!["revision".into()],
    };
    let expected = EvaluationCriterion::Judgement {
        criterion: "independent design review".into(),
        scope: Some(scope.clone()),
    };
    let encoded = serde_json::to_value(&expected).unwrap();
    let decoded: EvaluationCriterion = serde_json::from_value(encoded.clone()).unwrap();
    assert_eq!(serde_json::to_value(decoded).unwrap(), encoded);
    let legacy = json!({"kind":"judgement","criterion":"independent design review"});
    let decoded: EvaluationCriterion = serde_json::from_value(legacy.clone()).unwrap();
    assert_eq!(serde_json::to_value(decoded).unwrap(), legacy);
    let retained = RetainedJudgement {
        schema: RETAINED_JUDGEMENT_SCHEMA.into(),
        criterion: "independent design review".into(),
        scope,
        reviewer_identity: "board/reviewer-17".into(),
        reviewed_at: "2026-10-25T11:59:30Z".into(),
        verdict: JudgementVerdict::Supported,
        reasons: vec!["Review retained with explicit limitations".into()],
        provenance: strict_json_exact(
            br#"{"$serde_json::private::Number":"literal","artifact":90071992547409931234567891}"#,
            Default::default(),
        )
        .unwrap(),
    };
    retained.validate().unwrap();
    let encoded = serde_json::to_vec(&retained).unwrap();
    let decoded: RetainedJudgement = serde_json::from_slice(&encoded).unwrap();
    assert_eq!(decoded, retained);
    jsonschema::validator_for(&retained_judgement_schema())
        .unwrap()
        .validate(&serde_json::to_value(retained).unwrap())
        .unwrap();
}

#[test]
fn dynamic_witness_wire_keeps_literal_compatibility_and_references_are_validated() {
    for value in [
        json!(true),
        Value::Null,
        json!({"$serde_json::private::Number":"literal"}),
    ] {
        let old = json!({"pointer":"/match","equals":value});
        let decoded: WitnessPredicate = serde_json::from_value(old.clone()).unwrap();
        assert_eq!(serde_json::to_value(decoded).unwrap(), old);
    }
    let dynamic = json!({"pointer":"/uid","equals_output":{"step_id":"resource","pointer":"/uid"}});
    let decoded: WitnessPredicate = serde_json::from_value(dynamic.clone()).unwrap();
    assert_eq!(serde_json::to_value(decoded).unwrap(), dynamic);
}

#[test]
fn whole_value_projection_is_additive_to_the_legacy_projection_wire() {
    for kind in ["project", "project_value"] {
        let encoded = json!({"kind":kind,"fields":{"first_uid":"/0/uid"}});
        let decoded: DerivationOperation = serde_json::from_value(encoded.clone()).unwrap();
        assert_eq!(serde_json::to_value(decoded).unwrap(), encoded);
    }
}

#[test]
fn unsupported_execution_constructs_are_rejected_before_execution() {
    let unsupported_witness = json!({"pointer":"/uid","equals":true,"equals_future_output":{"step_id":"resource","pointer":"/uid"}});
    assert!(
        serde_json::from_value::<WitnessPredicate>(unsupported_witness)
            .unwrap_err()
            .to_string()
            .contains("unknown field")
    );
    let unsupported_operation = json!({"kind":"project_future_value","fields":{"uid":"/uid"}});
    assert!(
        serde_json::from_value::<DerivationOperation>(unsupported_operation)
            .unwrap_err()
            .to_string()
            .contains("unknown variant")
    );
    let unsupported_plan = json!({"schema":PLAN_SCHEMA,"acquisitions":[],"derivations":[],"evaluations":[],"future_semantics":true});
    assert!(
        serde_json::from_value::<EvidenceExecutionPlan>(unsupported_plan)
            .unwrap_err()
            .to_string()
            .contains("unknown field")
    );
}

#[test]
fn original_literal_plan_decodes_with_defaults_and_keeps_literal_criteria() {
    let value = json!({
        "schema": PLAN_SCHEMA,
        "acquisitions": [{
            "id": "measurement",
            "source": {
                "provider_id": "provider", "capability_id": "read", "capability_version": "1",
                "input_schema_sha256": "a".repeat(64), "output_schema_sha256": "b".repeat(64),
                "qualified_contract_sha256": "c".repeat(64), "configuration_sha256": "d".repeat(64),
                "authentication_scope_sha256": "e".repeat(64), "scope": {}
            },
            "arguments": {}
        }],
        "derivations": [],
        "evaluations": [{
            "id": "accept", "requirement_id": "requirement",
            "inputs": [{"step_id": "measurement", "pointer": ""}],
            "criterion": {"kind": "equals", "expected": true}
        }]
    });
    let decoded: EvidenceExecutionPlan = serde_json::from_value(value).unwrap();
    decoded.validate().unwrap();
    assert_eq!(decoded.scheduling.maximum_concurrent_sources, 4);
    assert!(decoded.acquisitions[0].max_age_seconds.is_none());
    assert!(matches!(
        decoded.evaluations[0].criterion,
        EvaluationCriterion::Equals {
            expected: Value::Bool(true)
        }
    ));
    assert_eq!(
        decoded.evaluations[0].required_coverage,
        RequiredCoverage::Unspecified
    );
    jsonschema::validator_for(&plan_schema())
        .unwrap()
        .validate(&serde_json::to_value(decoded).unwrap())
        .unwrap();
}
