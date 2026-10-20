use eal_aspic::{export_aspic_view, solve_aspic};
use eal_core::ExecutionLimits;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

fn reference() -> Value {
    serde_json::from_str(include_str!(
        "../../../fixtures/aspic/export-reference.json"
    ))
    .unwrap()
}

#[test]
fn complete_views_and_forged_result_rejections_match_pinned_python() {
    let corpus = reference();
    assert_eq!(
        corpus["source_revision"],
        "e2d6f346a37f92483b9f9a13923b98feb053447b"
    );
    let schema = include_bytes!("../../../contracts/schemas/aspic-view.schema.json");
    assert_eq!(
        corpus["view_schema_sha256"],
        format!("{:x}", Sha256::digest(schema))
    );
    for case in corpus["cases"].as_array().unwrap() {
        let limits: ExecutionLimits = serde_json::from_value(
            case.get("host_limits")
                .cloned()
                .unwrap_or_else(|| json!({})),
        )
        .unwrap();
        let actual = export_aspic_view(&case["input"], &limits);
        if let Some(expected) = case.get("view") {
            assert_eq!(
                &actual.unwrap_or_else(|e| panic!("{}: {e}", case["name"])),
                expected,
                "{}",
                case["name"]
            );
        } else {
            let error = actual.expect_err(case["name"].as_str().unwrap());
            assert!(
                error
                    .to_lowercase()
                    .contains(&case["error_match"].as_str().unwrap().to_lowercase()),
                "{}: {error}; source: {}",
                case["name"],
                case["error"]
            );
        }
    }
}

#[test]
fn export_is_detached_and_budget_capture_can_only_tighten_host_policy() {
    let corpus = reference();
    let input = &corpus["cases"][0]["input"];
    let mut view = export_aspic_view(input, &ExecutionLimits::default()).unwrap();
    view["arguments"][0]["direct_subarguments"]
        .as_array_mut()
        .unwrap()
        .push(json!("edited"));
    assert_ne!(
        view["arguments"][0]["direct_subarguments"],
        input["formal"]["arguments"][0]["direct_subarguments"]
    );
    let mut smaller = input.clone();
    smaller["source_map"]["execution_limits"]["formal_arguments"] = json!(64);
    let view = export_aspic_view(&smaller, &ExecutionLimits::default()).unwrap();
    assert_eq!(view["execution_limits"]["formal_arguments"], 64);
    let host = ExecutionLimits {
        formal_arguments: 63,
        ..ExecutionLimits::default()
    };
    assert!(
        export_aspic_view(&smaller, &host)
            .unwrap_err()
            .contains("host policy")
    );
}

#[test]
fn fixed_input_byte_boundary_uses_python_json_spacing_and_utf8() {
    let limit = eal_aspic::export::MAX_INPUT_BYTES;
    let mut input = json!({"padding":"x".repeat(limit-15)});
    let limits = ExecutionLimits::default();
    assert_eq!(
        export_aspic_view(&input, &limits).unwrap_err(),
        "Export requires a theory and its formal result"
    );
    input["padding"] = json!("x".repeat(limit - 14));
    assert!(
        export_aspic_view(&input, &limits)
            .unwrap_err()
            .contains("4 MiB")
    );
    input["padding"] = json!("🧪".repeat((limit - 15) / 4 + 1));
    assert!(
        export_aspic_view(&input, &limits)
            .unwrap_err()
            .contains("4 MiB")
    );
}

#[test]
fn hard_json_depth_is_checked_before_caller_data_is_cloned() {
    let mut value = json!(null);
    for _ in 0..200 {
        value = json!([value]);
    }
    let limits = ExecutionLimits {
        json_depth: usize::MAX,
        ..ExecutionLimits::default()
    };
    assert!(
        export_aspic_view(&value, &limits)
            .unwrap_err()
            .contains("bounded JSON")
    );
}

#[test]
fn method_output_envelope_limit_applies_with_raised_graph_budgets() {
    let length = 600;
    let theory = json!({
        "premises":[{"atom":"p0","kind":"axiom"}],
        "rules":(0..length).map(|i|json!({"id":format!("r{i}"),"kind":"strict", "antecedents":[format!("p{i}")],"consequent":format!("p{}",i+1)})).collect::<Vec<_>>(),
        "contraries":[],"goal":format!("p{length}")
    });
    let limits = ExecutionLimits {
        formal_rules: length,
        formal_atoms: length + 1,
        formal_arguments: length + 1,
        json_nodes: 1_000_000,
        ..ExecutionLimits::default()
    };
    let formal = solve_aspic(&json!({"theory":theory}), &limits).unwrap();
    let input = json!({"theory":theory,"formal":formal});
    let error = export_aspic_view(&input, &limits).unwrap_err();
    assert!(
        error.contains("Method output exceeds byte limit"),
        "{error}"
    );
}
