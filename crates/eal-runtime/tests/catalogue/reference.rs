//! Exact portable metadata projection; only named clock/backend result fields
//! are normalized. Literal context fields retain their full JSON identity.
use super::*;
use std::collections::BTreeMap;
fn resolve(value: &Value, saved: &BTreeMap<String, Value>) -> Value {
    match value {
        Value::String(text) if text.starts_with('$') => {
            let (name, field) = text[1..].split_once('.').unwrap();
            saved[name][field].clone()
        }
        Value::Array(values) => {
            Value::Array(values.iter().map(|value| resolve(value, saved)).collect())
        }
        Value::Object(values) if values.len() == 1 && values.contains_key("digest") => json!(
            canonical::canonical_digest(&values["digest"], &ExecutionLimits::default()).unwrap()
        ),
        Value::Object(values) => Value::Object(
            values
                .iter()
                .map(|(key, value)| (key.clone(), resolve(value, saved)))
                .collect(),
        ),
        _ => value.clone(),
    }
}
fn entry(value: &mut Value) {
    let Some(object) = value.as_object_mut() else {
        return;
    };
    object.remove("registered_at");
    object.remove("method_registry_fingerprint");
}
fn view(op: &str, mut value: Value) -> Value {
    match op {
        "register" | "get" | "get_cached" | "get_revision" => entry(&mut value),
        "list" | "find" | "revisions" => {
            for value in value.as_array_mut().unwrap() {
                entry(value);
            }
        }
        "register_tree" => {
            for value in value["registered"].as_array_mut().unwrap() {
                entry(value);
            }
            for value in value["rejected"].as_array_mut().unwrap() {
                if value["reason"].as_str().is_some_and(|text| {
                    text.starts_with("Registered source failed EAL validation:")
                }) {
                    value["reason"] = json!("Registered source failed EAL validation");
                }
            }
        }
        "runs" => {
            for item in value["collections"].as_array_mut().unwrap() {
                item.as_object_mut().unwrap().remove("created_at");
            }
            for item in value["assessments"].as_array_mut().unwrap() {
                item.as_object_mut().unwrap().remove("created_at");
                item.as_object_mut()
                    .unwrap()
                    .remove("method_registry_fingerprint");
            }
        }
        _ => {}
    }
    value
}
fn perform(
    workspace: &Workspace,
    catalogue: &mut WorkspaceKnowledgeCatalogue,
    step: &Value,
) -> Result<Value, String> {
    let op = step["op"].as_str().unwrap();
    let id = step["entry_id"].as_str().unwrap_or("");
    let limit = step["limit"].as_u64().unwrap_or(50) as usize;
    match op {
        "register" => {
            let claims = step.get("claims").map(|claims| {
                claims
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|claim| claim.as_str().unwrap().to_owned())
                    .collect::<Vec<_>>()
            });
            catalogue.register(
                step["path"].as_str().unwrap(),
                RegisterOptions {
                    entry_id: step["entry_id"].as_str(),
                    context: step.get("context"),
                    claims: claims.as_deref(),
                    label: step["label"].as_str(),
                },
            )
        }
        "get" => catalogue.get(id, step["include_source"].as_bool().unwrap_or(false)),
        "get_cached" => {
            catalogue.get_with_refresh(id, step["include_source"].as_bool().unwrap_or(false), false)
        }
        "get_revision" => catalogue.get_revision(
            id,
            step["source_digest"].as_str().unwrap(),
            step["method_registry_fingerprint"].as_str(),
            step["include_source"].as_bool().unwrap_or(false),
        ),
        "revisions" => catalogue.revisions(id, limit).map(Value::Array),
        "list" => catalogue
            .list(limit, step["offset"].as_u64().unwrap_or(0) as usize)
            .map(Value::Array),
        "find" => catalogue
            .find(FindOptions {
                query: step["query"].as_str(),
                claim: step["claim"].as_str(),
                context: step.get("context"),
                path: step["path"].as_str(),
                limit,
            })
            .map(Value::Array),
        "runs" => catalogue.runs(id, step["source_digest"].as_str(), limit),
        "register_tree" => catalogue.register_tree(
            step["directory"].as_str().unwrap_or("."),
            step.get("context"),
            step["limit"].as_u64().unwrap_or(512) as usize,
        ),
        "write" => {
            workspace.write(
                step["path"].as_str().unwrap(),
                step["source"].as_str().unwrap(),
            );
            Ok(Value::Null)
        }
        "reload" => {
            *catalogue = workspace.catalogue();
            Ok(Value::Null)
        }
        "put" => catalogue
            .service()
            .store()
            .put(
                step["kind"].as_str().unwrap(),
                &step["payload"],
                step["record_id"].as_str(),
            )
            .map(Value::String),
        _ => panic!("Unsupported fixture operation {op}"),
    }
}
#[test]
fn all_pinned_catalogue_operations_match_with_the_documented_generated_id_fix() {
    let corpus: Value = serde_json::from_str(include_str!(
        "../../../../fixtures/runtime/catalogue-reference.json"
    ))
    .unwrap();
    assert_eq!(
        corpus["source_revision"],
        "e2d6f346a37f92483b9f9a13923b98feb053447b"
    );
    assert_eq!(corpus["cases"].as_array().unwrap().len(), 23);
    let mut failures = Vec::new();
    let mut total = 0;
    let mut fixes = 0;
    for case in corpus["cases"].as_array().unwrap() {
        let workspace = Workspace::new();
        for (path, source) in case["files"].as_object().unwrap() {
            workspace.write(path, source.as_str().unwrap());
        }
        let mut catalogue = workspace.catalogue();
        let mut saved = BTreeMap::new();
        for (index, step) in case["steps"].as_array().unwrap().iter().enumerate() {
            total += 1;
            let input = resolve(step, &saved);
            let op = step["op"].as_str().unwrap();
            let output = perform(&workspace, &mut catalogue, &input);
            if step["native_fix"] == "existing_generated_id_refresh" {
                assert_eq!(
                    step["expected"]["error"],
                    "Explicit entry_id must be a simple developer name"
                );
                let output = output.unwrap();
                assert_eq!(output["entry_id"], "nested/source.eal");
                assert_eq!(output["source"], case["files"]["nested/source.eal"]);
                fixes += 1;
                continue;
            }
            match (output, step["expected"].get("value")) {
                (Ok(output), Some(expected)) => {
                    if let Some(name) = step["save"].as_str() {
                        saved.insert(name.to_owned(), output.clone());
                    }
                    if matches!(op, "register" | "get" | "get_cached" | "get_revision") {
                        assert_eq!(
                            output["method_registry_fingerprint"],
                            catalogue.service().method_registry().fingerprint()
                        );
                        eal_core::validation::parse_time(output["registered_at"].as_str().unwrap())
                            .unwrap();
                    }
                    let expected = resolve(expected, &saved);
                    let actual = view(op, output);
                    let expected = view(op, expected);
                    if actual != expected {
                        failures.push(format!(
                            "{} step{index} {op}: expected {expected}, observed {actual}",
                            case["name"]
                        ));
                    }
                }
                (Err(error), None) => {
                    let expected = step["expected"]["error"].as_str().unwrap();
                    let matches =
                        if expected.starts_with("Registered source failed EAL validation:") {
                            error.starts_with("Registered source failed EAL validation:")
                        } else if step["expected"]["kind"] == "KeyError" {
                            eal_core::expressions::diagnostic_repr(
                                &json!(error),
                                catalogue.service().limits(),
                            )
                            .unwrap()
                                == expected
                        } else {
                            error == expected
                        };
                    if !matches {
                        failures.push(format!(
                            "{} step{index} {op}: expected error {expected}, observed {error}",
                            case["name"]
                        ));
                    }
                }
                (actual, expected) => failures.push(format!(
                    "{} step{index} {op}: expected {expected:?}, observed {actual:?}",
                    case["name"]
                )),
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    assert_eq!((total, fixes), (68, 1));
}
