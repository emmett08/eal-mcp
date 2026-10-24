//! Independent probes of the public formal/provenance and resource boundaries.
#[path = "../../../tests/native_case.rs"]
mod native_case;
use eal_aspic::{AspicError, CompileOptions, compile_eal_aspic, export_aspic_view, solve_aspic};
use eal_core::{ExecutionLimits, canonical, methods};
use serde_json::{Value, json};

fn ordinary(atom: &str) -> Value {
    json!({"atom":atom,"kind":"ordinary","rank":500})
}

fn bare_theory() -> Value {
    json!({"premises":[ordinary("p")],"rules":[],"contraries":[],"goal":"p"})
}

fn supplied(theory: Value, limits: &ExecutionLimits) -> Value {
    let formal = solve_aspic(&json!({"theory":&theory}), limits).unwrap();
    json!({"theory":theory,"formal":formal})
}

fn mapped() -> Value {
    let mut result = supplied(bare_theory(), &ExecutionLimits::default());
    result["source_map"] = json!({
        "theory_digest":result["formal"]["theory_sha256"],
        "goal":{"claim":"selected","atom":"p"},
        "claims":{"selected":{"atom":"p","statement":"Supplied statement","span":null}},
        "arguments":{},"assumptions":{},"objections":{},"evidence":{}
    });
    result
}

#[test]
fn no_extension_precedes_unconstructed_without_vacuous_acceptance() {
    let mut theory = json!({
        "premises":[ordinary("p"),ordinary("q"),ordinary("r")],
        "rules":[],"contraries":[
            {"attacker":"p","target":"q"},
            {"attacker":"q","target":"r"},
            {"attacker":"r","target":"p"}],
        "goal":"absent","options":{"semantics":"stable","query_mode":"sceptical"}
    });
    let result = solve_aspic(&json!({"theory":&theory}), &ExecutionLimits::default()).unwrap();
    assert_eq!(result["grounded_status"], "unconstructed");
    assert_eq!(result["query_status"], "no_extension");
    assert_eq!(result["extensions"], json!([]));
    assert_eq!(result["query_accepted"], false);
    assert_eq!(result["query_rejected"], false);
    theory["options"]["semantics"] = json!("preferred");
    let result = solve_aspic(&json!({"theory":theory}), &ExecutionLimits::default()).unwrap();
    assert_eq!(result["extensions"], json!([[]]));
    assert_eq!(result["query_status"], "unconstructed");
    assert_eq!(result["query_accepted"], false);
    assert_eq!(result["query_rejected"], false);
}

#[test]
fn witness_budget_counts_multiple_attacked_subarguments_per_edge() {
    let theory = json!({
        "premises":[ordinary("p"),ordinary("q")],
        "rules":[
            {"id":"rx","kind":"defeasible","antecedents":["p"],"consequent":"x","name":"use_x","rank":500},
            {"id":"ry","kind":"strict","antecedents":["x"],"consequent":"y"}],
        "contraries":[{"attacker":"q","target":"p"},{"attacker":"q","target":"x"}],
        "goal":"y"
    });
    let mut limits = ExecutionLimits {
        formal_defeats: 4,
        ..ExecutionLimits::default()
    };
    assert!(solve_aspic(&json!({"theory":&theory}), &limits).is_err());
    limits.formal_defeats = 5;
    let result = solve_aspic(&json!({"theory":theory}), &limits).unwrap();
    assert_eq!(result["defeats"].as_array().unwrap().len(), 5);
    assert_eq!(result["defeat_count"], 3);
    assert_eq!(result["query_status"], "rejected");
}

#[test]
fn exhausted_search_returns_an_error_instead_of_a_partial_extension() {
    let theory = json!({
        "premises":[ordinary("p"),ordinary("q")],"rules":[],
        "contraries":[{"attacker":"p","target":"q"},{"attacker":"q","target":"p"}],
        "goal":"p","options":{"semantics":"preferred"}
    });
    let limits = ExecutionLimits {
        extension_search: 1,
        ..ExecutionLimits::default()
    };
    let error = solve_aspic(&json!({"theory":&theory}), &limits).unwrap_err();
    assert!(
        error.to_string().to_lowercase().contains("budget"),
        "{error}"
    );
    assert!(matches!(error, AspicError::Incomplete(_)));
    let dispatched = methods::assess_mode_with_limits(
        eal_aspic::METHOD,
        &[json!({"id":"formal","kind":"aspic_theory","value":{"theory":theory}})],
        &[],
        &eal_aspic::aspic_registry(),
        &limits,
    );
    assert_eq!(dispatched["status"], "incomplete");
    assert_eq!(dispatched["details"], json!({}));
    let limits = ExecutionLimits {
        formal_arguments: 1,
        ..ExecutionLimits::default()
    };
    let mut theory = bare_theory();
    theory["premises"]
        .as_array_mut()
        .unwrap()
        .push(ordinary("q"));
    assert!(solve_aspic(&json!({"theory":theory}), &limits).is_err());
}

#[test]
fn raised_argument_limits_survive_extension_masks_and_export_policy() {
    let theory = json!({
        "premises":(0..130).map(|i| ordinary(&format!("p{i}"))).collect::<Vec<_>>(),
        "rules":[],"contraries":[],"goal":"p129",
        "options":{"semantics":"stable"}
    });
    assert!(solve_aspic(&json!({"theory":&theory}), &ExecutionLimits::default()).is_err());
    let limits = ExecutionLimits {
        formal_premises: 130,
        formal_arguments: 130,
        formal_atoms: 130,
        ..ExecutionLimits::default()
    };
    let result = supplied(theory, &limits);
    assert_eq!(result["formal"]["argument_count"], 130);
    assert_eq!(result["formal"]["query_status"], "accepted");
    assert_eq!(
        result["formal"]["extensions"][0].as_array().unwrap().len(),
        130
    );
    let view = export_aspic_view(&result, &limits).unwrap();
    assert_eq!(view["arguments"].as_array().unwrap().len(), 130);
    assert!(export_aspic_view(&result, &ExecutionLimits::default()).is_err());
}

#[test]
fn exporter_recomputes_every_formal_field_and_detaches_output() {
    let limits = ExecutionLimits::default();
    let original = supplied(bare_theory(), &limits);
    for pointer in [
        "/formal/query_accepted",
        "/formal/query_rejected",
        "/formal/grounded_accepted",
        "/formal/argument_count",
        "/formal/defeat_count",
        "/formal/extensions",
        "/formal/arguments/0/label",
        "/formal/arguments/0/strength",
        "/formal/arguments/0/direct_subarguments",
        "/formal/defeats",
    ] {
        let mut changed = original.clone();
        let value = changed.pointer_mut(pointer).unwrap();
        *value = match pointer {
            "/formal/query_rejected" => json!(true),
            "/formal/query_accepted" | "/formal/grounded_accepted" => json!(false),
            "/formal/argument_count" => json!(2),
            "/formal/defeat_count" => json!(1),
            "/formal/extensions" => json!([]),
            "/formal/arguments/0/label" => json!("out"),
            "/formal/arguments/0/strength" => json!(499),
            "/formal/arguments/0/direct_subarguments" => json!(["A0"]),
            "/formal/defeats" => {
                json!([{"attacker":"A0","target":"A0","subargument":"A0","kind":"undermine"}])
            }
            _ => unreachable!(),
        };
        assert!(
            export_aspic_view(&changed, &limits).is_err(),
            "accepted edited {pointer}"
        );
    }
    let mut view = export_aspic_view(&original, &limits).unwrap();
    view["arguments"][0]["direct_subarguments"] = json!(["edited"]);
    assert_eq!(
        original["formal"]["arguments"][0]["direct_subarguments"],
        json!([])
    );
}

#[test]
fn exporter_validates_unused_source_metadata_and_preserves_supplied_prose() {
    let limits = ExecutionLimits::default();
    let mut original = mapped();
    let prose = "</script><script>supplied provenance</script>";
    original["source_map"]["claims"]["selected"]["statement"] = json!(prose);
    original["source_map"]["extra_host_metadata"] = json!({"key":"allowed"});
    let view = export_aspic_view(&original, &limits).unwrap();
    assert_eq!(
        view["validation"],
        json!({"formal_result":"recomputed","provenance":"supplied"})
    );
    assert_eq!(view["arguments"][0]["authored_statement"], prose);
    for entry in [
        json!({"available":false,"availability_issues":["missing_observation","missing_observation"]}),
        json!({"available":false,"availability_issues":["invented_issue"]}),
        json!({"available":false,"availability_issues":["missing_observation"],"span":{"line":0,"column":1,"end_line":1,"end_column":1}}),
        json!({"available":false,"availability_issues":["missing_observation"],"identity":{"run_id":"r","data_digest":"G".repeat(64),"request_digest":"0".repeat(64),"collected_at":"t","tool_binding_digest":"0".repeat(64)}}),
    ] {
        let mut changed = original.clone();
        changed["source_map"]["evidence"]["unused"] = entry;
        assert!(export_aspic_view(&changed, &limits).is_err(), "{changed}");
    }
    let mut changed = original.clone();
    changed["source_map"]["claims"]["selected"]["span"] =
        json!({"line":1.0,"column":1.0,"end_line":1.0,"end_column":2.0});
    assert!(
        export_aspic_view(&changed, &limits).is_ok(),
        "JSON Schema integers include integral JSON numbers"
    );
}

#[test]
fn exporter_enforces_unicode_lengths_and_every_snapshot_budget() {
    let limits = ExecutionLimits::default();
    let mut result = mapped();
    result["source_map"]["claims"]["selected"]["statement"] = json!("🦀".repeat(100_000));
    assert!(
        export_aspic_view(&result, &limits).is_ok(),
        "length counts Unicode scalars, not UTF-8 bytes"
    );
    result["source_map"]["claims"]["selected"]["statement"] = json!("🦀".repeat(100_001));
    assert!(export_aspic_view(&result, &limits).is_err());
    let mut result = mapped();
    let mut selected = limits.describe();
    selected["formal_construction"] = json!(limits.formal_construction + 1);
    result["source_map"]["execution_limits"] = selected;
    assert!(
        export_aspic_view(&result, &limits).is_err(),
        "even unused budget increases require host policy"
    );
    result["source_map"]["execution_limits"] = json!({"unknown_budget":1});
    assert!(export_aspic_view(&result, &limits).is_err());
}

#[test]
fn exporter_preflights_forged_depth_before_cloning_or_serialising() {
    std::thread::Builder::new()
        .stack_size(2 * 1024 * 1024)
        .spawn(|| {
            let limits = ExecutionLimits {
                json_depth: usize::MAX,
                json_nodes: usize::MAX,
                ..ExecutionLimits::default()
            };
            let mut result = supplied(bare_theory(), &ExecutionLimits::default());
            let mut nested = Value::Null;
            for _ in 0..5000 {
                nested = Value::Array(vec![nested]);
            }
            let mut fields = serde_json::Map::new();
            fields.insert("extra".into(), nested);
            let theory = Value::Object(fields);
            assert!(eal_aspic::validate_theory(&theory, &limits).is_err());
            // Move the deep value without recursively cloning it.
            let mut theory = theory;
            let nested = theory.as_object_mut().unwrap().remove("extra").unwrap();
            result["unused"] = nested;
            assert!(export_aspic_view(&result, &limits).is_err());
            std::mem::forget(result); // The constructing caller owns a forged recursive value's destructor.
        })
        .unwrap()
        .join()
        .unwrap();
}

#[test]
fn exporter_keeps_installed_contract_byte_caps_when_host_limits_are_raised() {
    let missing = format!("x{}", "a".repeat(63));
    let theory = json!({
        "premises":[ordinary("p")],
        "rules":(0..1100).map(|i| json!({
            "id":format!("r{i:04}{}","a".repeat(59)),"kind":"strict",
            "antecedents":[&missing],"consequent":format!("q{i:04}{}","a".repeat(59))
        })).collect::<Vec<_>>(),
        "contraries":[],"goal":"p"
    });
    let limits = ExecutionLimits {
        formal_rules: 1100,
        formal_atoms: 1102,
        ..ExecutionLimits::default()
    };
    assert!(
        canonical::canonical_json_bytes(&json!({"theory":&theory}), &limits)
            .unwrap()
            .len()
            > eal_aspic::aspic_contract().max_input_bytes
    );
    let result = supplied(theory, &limits);
    let error = export_aspic_view(&result, &limits).unwrap_err();
    assert!(error.contains("Method input exceeds byte limit"), "{error}");

    let theory = json!({
        "premises":[ordinary("p0")],
        "rules":(0..600).map(|i| json!({
            "id":format!("r{i}"),"kind":"strict",
            "antecedents":[format!("p{i}")],"consequent":format!("p{}",i+1)
        })).collect::<Vec<_>>(),
        "contraries":[],"goal":"p600"
    });
    let limits = ExecutionLimits {
        formal_rules: 600,
        formal_atoms: 601,
        formal_arguments: 601,
        json_nodes: 1_000_000,
        ..ExecutionLimits::default()
    };
    assert!(
        canonical::canonical_json_bytes(&json!({"theory":&theory}), &limits)
            .unwrap()
            .len()
            < eal_aspic::aspic_contract().max_input_bytes
    );
    let result = supplied(theory, &limits);
    let formal_bytes =
        canonical::canonical_json_bytes(&json!({"ok":true,"output":&result["formal"]}), &limits)
            .unwrap();
    assert!(formal_bytes.len() > eal_aspic::aspic_contract().max_output_bytes);
    let error = export_aspic_view(&result, &limits).unwrap_err();
    assert!(
        error.contains("Method output exceeds byte limit"),
        "{error}"
    );
}

// The bounded export representation includes one space per comma and colon.
fn spaced_json_size(value: &Value) -> usize {
    let mut extra = 0;
    let mut pending = vec![value];
    while let Some(value) = pending.pop() {
        match value {
            Value::Array(items) => {
                extra += items.len().saturating_sub(1);
                pending.extend(items);
            }
            Value::Object(items) => {
                extra += items.len() + items.len().saturating_sub(1);
                pending.extend(items.values());
            }
            _ => {}
        }
    }
    canonical::canonical_json_bytes(value, &ExecutionLimits::default())
        .unwrap()
        .len()
        + extra
}

#[test]
fn fixed_export_input_limit_counts_the_exact_wire_utf8_representation() {
    let limits = ExecutionLimits::default();
    let mut result = supplied(bare_theory(), &limits);
    result["padding"] = json!("");
    let available = eal_aspic::export::MAX_INPUT_BYTES - spaced_json_size(&result);
    result["padding"] = json!("x".repeat(available));
    assert_eq!(
        spaced_json_size(&result),
        eal_aspic::export::MAX_INPUT_BYTES
    );
    assert!(export_aspic_view(&result, &limits).is_ok());
    result["padding"] = json!("x".repeat(available + 1));
    let error = export_aspic_view(&result, &limits).unwrap_err();
    assert!(error.contains("4 MiB export input limit"), "{error}");
}

fn eal_case() -> Value {
    native_case::case()
}

#[test]
fn compiled_snapshot_getters_and_projections_are_detached() {
    let case = eal_case();
    let compiled = compile_eal_aspic(
        case["source"].as_str().unwrap(),
        &case["records"],
        CompileOptions::new("working", case["now"].as_str().unwrap(), &case["context"]),
    )
    .unwrap();
    let original = compiled.to_dict().unwrap();
    let mut theory = compiled.theory().unwrap();
    theory["goal"] = json!("forged");
    let mut mapping = compiled.source_map().unwrap();
    mapping["claims"]["working"]["statement"] = json!("Changed statement");
    let mut assessment = compiled.assessment().unwrap();
    assessment["claims"]["working"]["status"] = json!("out_of_scope");
    let mut projection = compiled.solve().unwrap();
    projection["source_map"]["goal"]["claim"] = json!("downstream");
    let mut limits = compiled.limits().unwrap();
    limits.formal_arguments = 1;
    assert_eq!(compiled.to_dict().unwrap(), original);
    assert_eq!(original["claim_status"], "supported");
    assert_eq!(original["claim"], "working");
    assert_ne!(compiled.limits().unwrap(), limits);
    let backend = &original["source_map"]["backend"];
    assert_eq!(backend["method"], "argumentation/aspic/2");
    assert_eq!(
        backend["module_digest"],
        eal_aspic::registry::module_identity()
    );
    assert_eq!(
        backend["contract_digest"],
        canonical::canonical_digest(
            &eal_aspic::aspic_contract().describe(),
            &ExecutionLimits::default()
        )
        .unwrap()
    );
}

#[test]
fn compiled_snapshot_preserves_original_observation_age_and_binding_correspondence() {
    let case = eal_case();
    let source = case["source"].as_str().unwrap();
    let at_boundary = compile_eal_aspic(
        source,
        &case["records"],
        CompileOptions::new("working", "2026-10-24T12:01:00Z", &case["context"]),
    )
    .unwrap()
    .to_dict()
    .unwrap();
    assert_eq!(at_boundary["claim_status"], "supported");
    assert_eq!(
        at_boundary["source_map"]["evidence"]["observed"]["identity"]["collected_at"],
        "2026-10-24T12:00:00Z"
    );
    let stale = compile_eal_aspic(
        source,
        &case["records"],
        CompileOptions::new("working", "2026-10-24T12:01:00.000001Z", &case["context"]),
    )
    .unwrap()
    .to_dict()
    .unwrap();
    let unavailable = &stale["source_map"]["evidence"]["observed"];
    assert_eq!(unavailable["available"], false);
    assert_eq!(unavailable["identity"], Value::Null);
    assert_eq!(
        unavailable["availability_issues"],
        json!(["stale_observation"])
    );
    assert_eq!(stale["claim_status"], "unsupported");
    assert_eq!(stale["formal"]["query_status"], "unconstructed");
    assert!(
        !stale["theory"]["premises"]
            .as_array()
            .unwrap()
            .iter()
            .any(|premise| premise["atom"] == unavailable["atom"])
    );
    let bindings = json!({"observed":"1".repeat(64)});
    let mut options =
        CompileOptions::new("working", case["now"].as_str().unwrap(), &case["context"]);
    options.binding_digests = Some(&bindings);
    let rebound = compile_eal_aspic(source, &case["records"], options)
        .unwrap()
        .to_dict()
        .unwrap();
    assert_eq!(
        rebound["source_map"]["evidence"]["observed"]["availability_issues"],
        json!(["invalid_observation"])
    );
    assert_eq!(rebound["claim_status"], "unsupported");
    assert_ne!(rebound["snapshot_digest"], at_boundary["snapshot_digest"]);
}

#[test]
fn unusable_observations_still_contribute_to_snapshot_identity() {
    let case = eal_case();
    let mut outputs = Vec::new();
    for marker in [1, 2] {
        let mut records = case["records"].clone();
        records["observed"]["value"] = json!({"passed":false,"marker":marker});
        records["observed"]["data_digest"] = json!(
            canonical::canonical_digest(&records["observed"]["value"], &ExecutionLimits::default())
                .unwrap()
        );
        let output = compile_eal_aspic(
            case["source"].as_str().unwrap(),
            &records,
            CompileOptions::new("working", case["now"].as_str().unwrap(), &case["context"]),
        )
        .unwrap()
        .to_dict()
        .unwrap();
        assert_eq!(
            output["source_map"]["evidence"]["observed"]["identity"],
            Value::Null
        );
        assert_eq!(
            output["source_map"]["evidence"]["observed"]["availability_issues"],
            json!(["predicate_not_met"])
        );
        assert_eq!(output["claim_status"], "unsupported");
        outputs.push(output);
    }
    assert_eq!(outputs[0]["theory"], outputs[1]["theory"]);
    assert_ne!(outputs[0]["snapshot_digest"], outputs[1]["snapshot_digest"]);
    assert_ne!(
        outputs[0]["source_map"]["evidence"]["observed"]["record_digest"],
        outputs[1]["source_map"]["evidence"]["observed"]["record_digest"]
    );
}

#[test]
fn incomplete_local_method_never_creates_a_compiled_formal_snapshot() {
    fn unfinished(_: &Value, _: &ExecutionLimits) -> Result<Value, methods::MethodError> {
        Err(methods::MethodError::Incomplete(
            "Independent review computation exhausted".into(),
        ))
    }
    let contract = methods::MethodContract::native(
        "review/unfinished/1",
        "test",
        json!({"type":"object","properties":{"passed":{"type":"boolean"}},"required":["passed"],"additionalProperties":false}),
        json!({"type":"object","properties":{},"required":[],"additionalProperties":false}),
        json!({"type":"object","properties":{"usable":{"type":"boolean"}},"required":["usable"],"additionalProperties":false}),
        methods::NativeMethodIdentity {
            version: "review/1".into(),
            artifact_sha256: "a".repeat(64),
        },
        unfinished,
    );
    let registry = methods::default_registry().with_method(contract).unwrap();
    let case = eal_case();
    let source = case["source"]
        .as_str()
        .unwrap()
        .replacen(
            "method \"structured/1\"",
            "method \"review/unfinished/1\"\n  require usable == true",
            1,
        )
        .replace(
            "argument propagate = [premises working] via measured => downstream\n",
            "",
        );
    // The record must correspond to the exact changed source before the local method can run.
    let program =
        eal_core::composition::compile(&source, None, "", ExecutionLimits::default()).unwrap();
    let mut records = case["records"].clone();
    records["observed"]["source_digest"] = json!(program.source_digest);
    let mut options =
        CompileOptions::new("working", case["now"].as_str().unwrap(), &case["context"]);
    options.registry = Some(&registry);
    let error = compile_eal_aspic(&source, &records, options).unwrap_err();
    assert!(matches!(error, AspicError::Incomplete(_)), "{error}");
    assert!(
        error
            .to_string()
            .contains("Local EAL assessment is incomplete"),
        "{error}"
    );
}
