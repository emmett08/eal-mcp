//! Independent malformed designs test schema guidance and semantic admission together.
use eal_collector_contracts::{capability::*, execution::*};
use serde_json::{Value, json};
use std::collections::BTreeMap;

fn contract() -> SupportContract {
    serde_json::from_value(json!({
        "schema":"eal.support-contract/1","support_type":"source_retrieval",
        "scope":{"kind":"resource","subject":"a named resource","selectors":{"cluster":"dev"},"identity_dimensions":["cluster"]},
        "acceptable_methods":["kubernetes.read/1"],"quantities":[{"name":"record","unit":"JSON"}],
        "logical_strength":"existential_witness","population":null,
        "time":{"historical_interval":null,"max_age_seconds":null,"max_uncertainty_seconds":null,"require_original_observation_time":true},
        "provenance":{"required_authorities":["cluster-api"],"transformation_history":true},"expected_output_schema_sha256":null
    })).unwrap()
}

#[test]
fn contract_schema_and_runtime_reject_wrong_identity_and_empty_methods_together() {
    let schema = serde_json::to_value(schemars::schema_for!(SupportContract)).unwrap();
    let validator = jsonschema::validator_for(&schema).unwrap();
    let good = contract();
    validator
        .validate(&serde_json::to_value(&good).unwrap())
        .unwrap();
    let mut bad = good.clone();
    bad.schema = "engarg.support-contract/1".into();
    bad.acceptable_methods.clear();
    let errors = bad.diagnostics();
    assert_eq!(errors.len(), 2);
    assert_eq!(errors[0].path, "/schema");
    assert_eq!(errors[0].code, "support_contract_schema");
    assert!(errors[0].message.contains("eal.support-contract/1"));
    assert!(errors[0].message.contains("engarg.support-contract/1"));
    assert_eq!(errors[1].path, "/acceptable_methods");
    assert!(bad.validate().unwrap_err().contains("received 0 entries"));
    assert_eq!(
        validator
            .iter_errors(&serde_json::to_value(bad).unwrap())
            .count(),
        2
    );
    for methods in [
        vec!["not-versioned".to_string()],
        vec!["api.get/1".to_string(); 2],
        vec!["api.get/1".to_string(); 129],
    ] {
        let mut bad = good.clone();
        bad.acceptable_methods = methods;
        assert!(bad.validate().is_err());
        assert!(!validator.is_valid(&serde_json::to_value(bad).unwrap()));
    }
}

fn discovery() -> DependencyRole {
    DependencyRole::Discovery {
        scope_bindings: BTreeMap::from([("/cluster".into(), "/cluster".into())]),
    }
}

#[test]
fn discovery_mapping_preserves_every_source_selector_and_shared_identity() {
    let role = discovery();
    role.validate_scopes(
        &json!({"cluster":"dev"}),
        &json!({"cluster":"dev","resource":"widgets"}),
    )
    .unwrap();
    assert!(
        role.validate_scopes(&json!({"cluster":"prod"}), &json!({"cluster":"dev"}))
            .is_err()
    );
    assert!(
        role.validate_scopes(
            &json!({"cluster":"dev","tenant":"red"}),
            &json!({"cluster":"dev"})
        )
        .is_err()
    );
    let renamed = DependencyRole::Discovery {
        scope_bindings: BTreeMap::from([("/cluster".into(), "/origin_cluster".into())]),
    };
    renamed
        .validate_scopes(&json!({"cluster":"dev"}), &json!({"origin_cluster":"dev"}))
        .unwrap();
    // A valid renamed field does not hide a contradictory common selector.
    assert!(
        renamed
            .validate_scopes(
                &json!({"cluster":"dev"}),
                &json!({"origin_cluster":"dev","cluster":"prod"})
            )
            .is_err()
    );
    for bindings in [
        BTreeMap::new(),
        BTreeMap::from([("".into(), "".into())]),
        BTreeMap::from([("/cluster".into(), "/absent".into())]),
    ] {
        assert!(
            DependencyRole::Discovery {
                scope_bindings: bindings
            }
            .validate_scopes(&json!({"cluster":"dev"}), &json!({"cluster":"dev"}))
            .is_err()
        );
    }
}

#[test]
fn legacy_binding_wire_and_identity_remain_unchanged() {
    let old = json!({"output":{"step_id":"search","pointer":"/ids"},"argument_pointer":"/id"});
    let mut binding: DependencyBinding = serde_json::from_value(old.clone()).unwrap();
    assert!(binding.role.is_selection());
    assert_eq!(serde_json::to_value(&binding).unwrap(), old);
    assert_eq!(
        canonical_digest(&binding).unwrap(),
        canonical_digest(&old).unwrap()
    );
    let prior = canonical_digest(&binding).unwrap();
    binding.role = discovery();
    assert_ne!(canonical_digest(&binding).unwrap(), prior);
    jsonschema::validator_for(&plan_schema()).unwrap();
}

fn source(id: &str, scope: Value) -> Value {
    json!({"id":id,"source":{"provider_id":"api","capability_id":"read","capability_version":"1",
        "input_schema_sha256":"1".repeat(64),"output_schema_sha256":"2".repeat(64),
        "qualified_contract_sha256":"3".repeat(64),"configuration_sha256":"4".repeat(64),
        "authentication_scope_sha256":"5".repeat(64),"scope":scope},"arguments":{"version":null},"bindings":[],"for_each":null})
}

fn plan() -> EvidenceExecutionPlan {
    let first = source("discovery", json!({"cluster":"dev"}));
    let mut second = source("read", json!({"cluster":"dev","resource":"widgets"}));
    second["bindings"] = json!([{"output":{"step_id":"projection","pointer":"/version"},"argument_pointer":"/version","role":discovery()}]);
    serde_json::from_value(json!({"schema":PLAN_SCHEMA,"acquisitions":[first,second],
        "derivations":[{"id":"projection","inputs":[{"step_id":"discovery","pointer":""}],"operation":{"kind":"project","fields":{"version":"/version"}}}],
        "evaluations":[]})).unwrap()
}

#[test]
fn execution_plan_checks_discovery_scope_through_only_pure_projections() {
    let valid = plan();
    valid.validate().unwrap();
    jsonschema::validator_for(&plan_schema())
        .unwrap()
        .validate(&serde_json::to_value(&valid).unwrap())
        .unwrap();
    let mut wrong_scope = valid.clone();
    wrong_scope.acquisitions[0].source.scope["cluster"] = json!("prod");
    assert!(
        wrong_scope
            .validate()
            .unwrap_err()
            .contains("unequal values")
    );
    let mut arithmetic = valid.clone();
    arithmetic.derivations[0].operation = DerivationOperation::Count;
    assert!(
        arithmetic
            .validate()
            .unwrap_err()
            .contains("only projection or extraction")
    );
    let mut selected = valid.clone();
    selected.acquisitions[0].bindings.push(DependencyBinding {
        output: OutputReference {
            step_id: "read".into(),
            pointer: "/record".into(),
        },
        argument_pointer: "/version".into(),
        role: DependencyRole::Selection,
    });
    assert!(
        selected
            .validate()
            .unwrap_err()
            .contains("selection-dependent")
    );
    let mut cycle = valid;
    cycle.derivations[0].inputs[0].step_id = "projection".into();
    assert!(cycle.validate().unwrap_err().contains("cyclic"));
}
