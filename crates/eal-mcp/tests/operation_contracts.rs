//! Independently authored request corpus shared by both public entry points.
use eal_mcp::{
    host,
    operations::{Exposure, OPERATIONS, Operation, OperationRequest},
};
use serde_json::{Value, json};

fn cases() -> Vec<(&'static str, Value)> {
    vec![
        ("describe", json!({})),
        ("format", json!({"source":"source"})),
        ("validate", json!({"source":"source"})),
        ("plan", json!({"source":"source","claim":"outcome"})),
        ("collect", json!({"source":"source","context":{}})),
        (
            "collect_claim",
            json!({"source":"source","context":{},"claim":"outcome"}),
        ),
        ("reason", json!({"source":"source","context":{}})),
        ("execute", json!({"source":"source","context":{}})),
        ("prepare", json!({"source":"source","context":{}})),
        (
            "compile_aspic",
            json!({"source":"source","context":{},"collection_id":"collection","goal":"outcome"}),
        ),
        ("explain", json!({"assessment_id":"assessment"})),
        ("packet", json!({"assessment_id":"assessment"})),
        ("sources", json!({})),
        ("find_claims", json!({})),
        (
            "assess_known",
            json!({"entry_id":"entry","claim":"outcome"}),
        ),
        (
            "get_known_packet",
            json!({"entry_id":"entry","assessment_id":"assessment","claim":"outcome"}),
        ),
        (
            "grounded",
            json!({"arguments":["a","b"],"attacks":[["a","b"]]}),
        ),
    ]
}
fn host_admits(name: &str, args: &Value) -> bool {
    let mut request = args.clone();
    request["operation"] = json!(name);
    host::parse_request(&request.to_string()).is_ok()
}
fn assert_admission(name: &str, args: &Value, expected: bool) {
    assert_eq!(
        OperationRequest::decode(&format!("eal_{name}"), args).is_ok(),
        expected,
        "SDK {name}: {args}"
    );
    assert_eq!(host_admits(name, args), expected, "text {name}: {args}");
}
#[test]
fn every_operation_uses_one_closed_typed_request_contract() {
    let cases = cases();
    assert_eq!(cases.len(), OPERATIONS.len());
    for (name, args) in cases {
        let operation = Operation::parse(name).unwrap();
        let decoded = OperationRequest::decode(name, &args).unwrap();
        assert_eq!(decoded.operation(), operation);
        assert_admission(name, &args, true);
        let schema = operation.schema();
        eal_mcp::guard::validate_schema(&args, &schema).unwrap();
        let mut unknown = args.clone();
        unknown["unrecognised"] = json!(true);
        assert_admission(name, &unknown, false);
        for field in operation.fields().required {
            let mut missing = args.clone();
            missing.as_object_mut().unwrap().remove(*field);
            assert_admission(name, &missing, false);
            let mut null = args.clone();
            null[*field] = Value::Null;
            assert_admission(name, &null, false);
            let mut wrong = args.clone();
            wrong[*field] = json!(42);
            assert_admission(name, &wrong, false);
            let mut empty = args.clone();
            empty[*field] = match &args[*field] {
                Value::String(_) => json!(""),
                Value::Object(_) => json!({}),
                Value::Array(_) => json!([]),
                _ => panic!("unexpected independently authored field type"),
            };
            let accepted = matches!(
                (name, *field),
                (_, "source" | "context") | ("grounded", "arguments" | "attacks")
            );
            assert_admission(name, &empty, accepted);
            assert_eq!(
                eal_mcp::guard::validate_schema(&empty, &schema).is_ok(),
                accepted
            );
        }
        let mut oversized = args.clone();
        oversized["oversized"] = json!("x".repeat(4 * 1024 * 1024 + 1));
        assert_admission(name, &oversized, false);
    }
}
#[test]
fn optional_fields_preserve_null_defaults_and_exact_numeric_admission() {
    for (name, field) in [
        ("reason", "collection_id"),
        ("reason", "now"),
        ("compile_aspic", "now"),
        ("compile_aspic", "preference"),
        ("explain", "claim"),
        ("packet", "claim"),
        ("find_claims", "query"),
        ("find_claims", "claim"),
        ("collect", "evidence_ids"),
    ] {
        let mut args = cases()
            .into_iter()
            .find(|(operation, _)| *operation == name)
            .unwrap()
            .1;
        args[field] = Value::Null;
        assert_admission(name, &args, true);
        args[field] = json!(42);
        assert_admission(name, &args, false);
    }
    for (name, field) in [
        ("compile_aspic", "semantics"),
        ("compile_aspic", "query_mode"),
        ("sources", "limit"),
        ("sources", "offset"),
        ("find_claims", "limit"),
        ("packet", "packet_limits"),
    ] {
        let mut args = cases()
            .into_iter()
            .find(|(operation, _)| *operation == name)
            .unwrap()
            .1;
        args[field] = Value::Null;
        assert_admission(name, &args, false);
    }
    for (name, field) in [
        ("reason", "collection_id"),
        ("explain", "claim"),
        ("packet", "claim"),
    ] {
        let mut args = cases()
            .into_iter()
            .find(|(operation, _)| *operation == name)
            .unwrap()
            .1;
        args[field] = json!("");
        assert_admission(name, &args, false);
    }
    assert_admission(
        "collect",
        &json!({"source":"source", "context":{}, "evidence_ids":[""]}),
        false,
    );
    assert_admission(
        "collect",
        &json!({"source":"source", "context":{}, "evidence_ids":[]}),
        true,
    );
    assert_admission("find_claims", &json!({"query":"", "claim":""}), true);
    for number in [
        "1.0",
        "1e0",
        "1E+0",
        "-0.0",
        "184467440737095516160000000000000000000000",
    ] {
        let args = eal_core::json::parse(
            &format!(r#"{{"limit":{number}}}"#),
            &eal_core::ExecutionLimits::default(),
        )
        .unwrap();
        assert_admission("sources", &args, false);
    }
    assert_admission("sources", &json!({"limit":100,"offset":10000}), true);
    assert_admission("sources", &json!({"limit":0}), false);
    assert_admission("sources", &json!({"limit":101}), false);
    assert_admission("sources", &json!({"offset":10001}), false);
    for budget in [
        json!({"max_bytes":4096,"max_statement_bytes":1}),
        json!({"max_bytes":65536,"max_statement_bytes":4096}),
        json!({}),
    ] {
        assert_admission(
            "packet",
            &json!({"assessment_id":"stored","packet_limits":budget}),
            true,
        );
    }
    for budget in [
        json!({"max_bytes":4096.0}),
        json!({"max_bytes":4095}),
        json!({"max_bytes":65537}),
        json!({"max_statement_bytes":0}),
        json!({"max_statement_bytes":null}),
        json!({"unknown":1}),
    ] {
        assert_admission(
            "packet",
            &json!({"assessment_id":"stored","packet_limits":budget}),
            false,
        );
    }
}
#[test]
fn identifiers_attack_pairs_and_literal_context_have_truthful_shapes() {
    for (name, field) in [
        ("plan", "claim"),
        ("explain", "assessment_id"),
        ("packet", "assessment_id"),
        ("collect_claim", "claim"),
        ("compile_aspic", "collection_id"),
        ("compile_aspic", "goal"),
        ("assess_known", "entry_id"),
        ("assess_known", "claim"),
        ("get_known_packet", "entry_id"),
        ("get_known_packet", "assessment_id"),
        ("get_known_packet", "claim"),
    ] {
        let mut args = cases()
            .into_iter()
            .find(|(operation, _)| *operation == name)
            .unwrap()
            .1;
        args[field] = json!("");
        assert_admission(name, &args, false);
    }
    for attacks in [
        json!([["a"]]),
        json!([["a", "b", "c"]]),
        json!([["a", 1]]),
        json!([null]),
    ] {
        assert_admission(
            "grounded",
            &json!({"arguments":[],"attacks":attacks}),
            false,
        );
    }
    let context = eal_core::json::parse(r#"{"$serde_json::private::Number":"literal","nested":{"$serde_json::private::RawValue":"text"},"large":12345678901234567890123456789012345678901234567890}"#, &eal_core::ExecutionLimits::default()).unwrap();
    let args = json!({"source":"source","context":context});
    let OperationRequest::Reason(request) = OperationRequest::decode("reason", &args).unwrap()
    else {
        panic!("wrong handler");
    };
    assert_eq!(&request.context, args["context"].as_object().unwrap());
    assert_admission("reason", &args, true);
}

#[cfg(unix)]
#[tokio::test]
async fn server_and_text_reject_each_invalid_shape_before_operation_effects() {
    use eal_core::{ExecutionLimits, methods::default_registry};
    use eal_mcp::server::{Server, ServerOptions};
    use eal_runtime::{
        catalogue::{RegisterOptions, WorkspaceKnowledgeCatalogue},
        process::Supervisor,
        scheduler::CollectionScheduler,
        service::{ReasoningService, ServiceConfig},
    };
    use std::{sync::Arc, time::Duration};
    use tokio_util::sync::CancellationToken;
    let root = std::env::temp_dir().join(format!("eal-operation-contracts-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    let service = Arc::new(
        ReasoningService::new(ServiceConfig {
            workspace: root.clone(),
            registry_path: None,
            database_path: None,
            method_registry: default_registry().clone(),
            method_executor: None,
            scheduler: CollectionScheduler::default(),
            supervisor: Supervisor::new("/unused-supervisor".into()),
            limits: ExecutionLimits::default(),
            lease_timeout: Duration::from_secs(3),
        })
        .unwrap(),
    );
    std::fs::write(root.join("argument.eal"), "language \"EAL/4\"\nenvironment lab {\n require true\n}\nclaim outcome {\n statement \"Outcome.\"\n environment lab\n}\n").unwrap();
    WorkspaceKnowledgeCatalogue::new(Arc::clone(&service))
        .unwrap()
        .register(
            "argument.eal",
            RegisterOptions {
                entry_id: Some("entry"),
                ..Default::default()
            },
        )
        .unwrap();
    let operator = Server::new(Arc::clone(&service), ServerOptions::default()).unwrap();
    let registered = Server::new(
        Arc::clone(&service),
        ServerOptions {
            known_entries: vec!["entry".into()],
            ..Default::default()
        },
    )
    .unwrap();
    let before = service.store().list(None, 1000).unwrap();
    for (name, args) in cases() {
        let operation = Operation::parse(name).unwrap();
        let server = if operation.exposure() == Exposure::Operator {
            &operator
        } else {
            &registered
        };
        let other = if operation.exposure() == Exposure::Operator {
            &registered
        } else {
            &operator
        };
        let exposed_error = other
            .call(
                &operation.tool_name(),
                args.clone(),
                CancellationToken::new(),
            )
            .await
            .unwrap_err();
        assert!(exposed_error.contains("EAL_OPERATION_EXPOSURE"));
        let mut invalids = vec![{
            let mut value = args.clone();
            value["unknown"] = json!(true);
            value
        }];
        for field in operation.fields().required {
            let mut missing = args.clone();
            missing.as_object_mut().unwrap().remove(*field);
            invalids.push(missing);
            let mut null = args.clone();
            null[*field] = Value::Null;
            invalids.push(null);
            let mut wrong = args.clone();
            wrong[*field] = json!(42);
            invalids.push(wrong);
        }
        for invalid in invalids {
            assert!(!host_admits(name, &invalid));
            let error = server
                .call(&operation.tool_name(), invalid, CancellationToken::new())
                .await
                .unwrap_err();
            assert!(error.contains("EAL_OPERATION"), "{name}: {error}");
        }
    }
    let lease = service.acquisition_lease().unwrap();
    let error = tokio::time::timeout(
        Duration::from_secs(1),
        registered.call(
            "eal_assess_known",
            json!({"entry_id":"foreign","claim":"outcome"}),
            CancellationToken::new(),
        ),
    )
    .await
    .expect("unexposed source waited for an acquisition lease")
    .unwrap_err();
    assert!(error.contains("not exposed"));
    drop(lease);
    assert_eq!(
        service.store().list(None, 1000).unwrap(),
        before,
        "invalid operations wrote retained state"
    );
    drop(operator);
    drop(registered);
    drop(service);
    std::fs::remove_dir_all(root).unwrap();
}
