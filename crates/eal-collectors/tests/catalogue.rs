use eal_collector_contracts::{Catalogue, CommandRegistry};
use eal_collectors::{Config, Request, catalogue};
use eal_core::{ExecutionLimits, composition, methods, validation};
use eal_runtime::acquisition::ToolRegistry;
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    process::{Command, Stdio},
};

#[test]
fn describe_is_a_pure_versioned_contract_and_matches_the_published_file() {
    let output = Command::new(env!("CARGO_BIN_EXE_eal-collector"))
        .arg("describe")
        .env_clear()
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let emitted =
        eal_runtime::acquisition::strict_json(std::str::from_utf8(&output.stdout).unwrap())
            .unwrap();
    let published = eal_runtime::acquisition::strict_json(catalogue::NATIVE_JSON).unwrap();
    assert_eq!(emitted, published);
    let parsed: Catalogue = serde_json::from_value(emitted.clone()).unwrap();
    parsed.validate().unwrap();
    let schema: Value = serde_json::from_str(include_str!(
        "../../../contracts/schemas/collector-catalogue.schema.json"
    ))
    .unwrap();
    jsonschema::validator_for(&schema)
        .unwrap()
        .validate(&emitted)
        .unwrap();
    let rejected = Command::new(env!("CARGO_BIN_EXE_eal-collector"))
        .args(["describe", "--executable", "/unexpected"])
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert!(!rejected.status.success());
    assert!(rejected.stdout.is_empty());
}

#[test]
fn native_descriptors_bind_to_current_cli_registry_and_eal_predicates() {
    for descriptor in &catalogue::native().collectors {
        let mut parameters = BTreeMap::from([
            ("inventory".into(), "/operator/inventory.json".into()),
            ("resource".into(), "applications.argoproj.io".into()),
            ("kind".into(), "Application".into()),
            ("namespace".into(), "argocd".into()),
            ("name".into(), "sync-operator".into()),
            ("repository".into(), "owner/repo".into()),
            ("commit".into(), "a".repeat(40)),
            ("url".into(), "https://tempo.example/api/search".into()),
            (
                "cluster_attribute".into(),
                "resource.k8s.cluster.name".into(),
            ),
            ("start_unix".into(), "1".into()),
            ("end_unix".into(), "2".into()),
        ]);
        parameters.retain(|key, _| descriptor.parameters.contains(key));
        let selected = BTreeMap::from([
            ("executable".into(), "/operator/external-cli".into()),
            (
                "supervisor".into(),
                "/operator/eal-command-supervisor".into(),
            ),
        ]);
        let binding = descriptor.binding(&parameters, &selected).unwrap();
        let config = Config::parse(binding.argv.iter().skip(1).cloned().collect()).unwrap();
        assert_eq!(config.provider().name(), descriptor.id);
        let registry = CommandRegistry {
            tools: BTreeMap::from([("reading_tool".into(), binding)]),
        };
        ToolRegistry::from_toml(&registry.to_toml().unwrap()).unwrap();
        let properties = descriptor.input_schema["properties"].as_object().unwrap();
        let mut input = json!({});
        for (key, property) in properties {
            input[key] = if property["type"] == "integer" {
                json!(parameters[key].parse::<u64>().unwrap())
            } else {
                json!(parameters[key])
            };
        }
        let validator = jsonschema::validator_for(&descriptor.input_schema).unwrap();
        validator.validate(&input).unwrap();
        let requested = json!({"evidence_id":"reading","environment":"scope","tool":"reading_tool","tool_version":descriptor.version,"input":input,"context":{}});
        Request::parse(&serde_json::to_vec(&requested).unwrap()).unwrap();
        let source = format!(
            "language \"EAL/3\"\nenvironment scope {{\n  require chosen == true\n}}\ntool reading_tool {{\n  version \"{}\"\n}}\nevidence reading {{\n  tool reading_tool\n  kind acquisition\n  environment scope\n  max_age 60\n  input {}\n  require {}\n}}\n",
            descriptor.version, input, descriptor.acquisition_predicate
        );
        let program = composition::compile(&source, None, "", ExecutionLimits::default()).unwrap();
        assert!(validation::validate(&program, methods::default_registry()).is_empty());
        jsonschema::validator_for(&descriptor.result_schema).unwrap();
    }
}

#[test]
fn native_input_schemas_reject_invalid_literal_selectors() {
    for descriptor in &catalogue::native().collectors {
        let validator = jsonschema::validator_for(&descriptor.input_schema).unwrap();
        match descriptor.id.as_str() {
            "kubernetes" => {
                for name in [
                    "--raw=/secrets".to_owned(),
                    "x".repeat(254),
                    "one,two".to_owned(),
                ] {
                    assert!(
                        !validator.is_valid(
                            &json!({"resource":"pods","namespace":"default","name":name})
                        )
                    );
                }
            }
            "github" => {
                for repository in ["owner/..", "../repo", "owner/repo/other"] {
                    assert!(
                        !validator
                            .is_valid(&json!({"repository":repository,"commit":"a".repeat(40)}))
                    );
                }
                assert!(!validator.is_valid(&json!({"repository":"owner/repo","commit":"main"})));
            }
            "tempo" => assert!(!validator.is_valid(&json!({"start_unix":"1","end_unix":2}))),
            other => panic!("Unexpected native collector {other}"),
        }
    }
}

#[cfg(unix)]
#[test]
fn a_custom_laboratory_catalogue_preserves_typed_input_and_measured_negatives() {
    use eal_collector_contracts::CollectorDescriptor;
    use eal_runtime::{acquisition::validate_envelope, process::Supervisor};
    use std::os::unix::fs::PermissionsExt;

    let descriptor: CollectorDescriptor = serde_json::from_value(json!({
        "id":"laboratory_calibration", "version":"1",
        "purpose":"Read calibration measurements for the selected channels and voltage.",
        "parameters":["channels","voltage"],
        "input_schema":{
            "type":"object", "additionalProperties":false,
            "required":["channels","voltage"],
            "properties":{
                "channels":{"type":"array","minItems":1,"items":{"type":"integer","minimum":1}},
                "voltage":{"type":"number","minimum":0}
            }
        },
        "result_schema":{
            "type":"object", "required":["collected","calibrated","reading"],
            "properties":{"collected":{"type":"boolean"},"calibrated":{"type":"boolean"},"reading":{"type":"number"}}
        },
        "command":{
            "argv":["{operator.executable}"], "inherit_env":[],
            "timeout_seconds":2, "max_output_bytes":4096, "parallel_safe":false
        },
        "acquisition_predicate":"collected == true", "constraints":[]
    })).unwrap();
    descriptor.validate().unwrap();
    let input = json!({"channels":[1,2],"voltage":1.25});
    let input_validator = jsonschema::validator_for(&descriptor.input_schema).unwrap();
    input_validator.validate(&input).unwrap();
    assert!(!input_validator.is_valid(&json!({"channels":"[1,2]","voltage":"1.25"})));
    let parameters = BTreeMap::from([
        ("channels".into(), "[1,2]".into()),
        ("voltage".into(), "1.25".into()),
    ]);
    assert!(descriptor.binding(&parameters, &BTreeMap::new()).is_err());

    let workspace =
        std::env::temp_dir().join(format!("eal-custom-catalogue-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir(&workspace).unwrap();
    let executable = workspace.join("laboratory-collector");
    std::fs::write(&executable, "#!/bin/sh\nIFS= read -r request\nprintf '%s' \"$request\" > observed-request.json\nprintf '%s' '{\"value\":{\"collected\":true,\"calibrated\":false,\"reading\":1.25}}'\n").unwrap();
    std::fs::set_permissions(&executable, std::fs::Permissions::from_mode(0o700)).unwrap();
    let binding = descriptor
        .binding(
            &parameters,
            &BTreeMap::from([("executable".into(), executable.to_str().unwrap().into())]),
        )
        .unwrap();
    let registry = ToolRegistry::from_toml(
        &CommandRegistry {
            tools: BTreeMap::from([("measurement".into(), binding)]),
        }
        .to_toml()
        .unwrap(),
    )
    .unwrap();
    let context = json!({"rig":"bench-1"});
    let request = json!({"evidence_id":"calibration","environment":"laboratory","tool":"measurement","tool_version":"1","input":input,"context":context});
    let acquisition =
        json!({"tool":"measurement","tool_version":"1","input":input,"context":context});
    let supervisor = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("eal-command-supervisor");
    assert!(
        supervisor.is_file(),
        "Build runtime companion binaries before this test"
    );
    let result = registry
        .acquire(
            registry.binding_for("measurement", "1").unwrap(),
            &request,
            &workspace,
            &[42; 32],
            &Supervisor::new(supervisor),
            None,
        )
        .unwrap();
    assert!(result.error.is_none(), "{:?}", result.error);
    let received: Value =
        serde_json::from_slice(&std::fs::read(workspace.join("observed-request.json")).unwrap())
            .unwrap();
    assert_eq!(received, request);
    let observed = validate_envelope(&result.stdout, false, &context, &acquisition).unwrap();
    jsonschema::validator_for(&descriptor.result_schema)
        .unwrap()
        .validate(&observed["value"])
        .unwrap();
    assert_eq!(observed["value"]["collected"], true);
    assert_eq!(observed["value"]["calibrated"], false);
    assert_eq!(observed["value"]["reading"], 1.25);
    std::fs::remove_dir_all(workspace).unwrap();
}

#[test]
fn explicit_native_v2_selection_advertises_and_binds_both_resource_scopes() {
    let output = Command::new(env!("CARGO_BIN_EXE_eal-collector"))
        .args(["describe", "--catalogue-version", "2"])
        .env_clear()
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let advertised: Value = serde_json::from_slice(&output.stdout).unwrap();
    let expected: Value = serde_json::from_str(catalogue::NATIVE_V2_JSON).unwrap();
    assert_eq!(advertised, expected);
    let native = catalogue::native_version("2").unwrap();
    native.validate().unwrap();
    assert!(catalogue::native_version("999").is_err());
    assert_eq!(catalogue::native().collectors[0].version, "1");
    for descriptor in &native.collectors {
        assert_eq!(descriptor.version, "2");
        let scope = if descriptor.id == "kubernetes_cluster" {
            "cluster"
        } else {
            "namespaced"
        };
        let mut parameters = BTreeMap::from([
            ("inventory".into(), "/operator/inventory.json".into()),
            ("resource".into(), "configmaps".into()),
            ("kind".into(), "ConfigMap".into()),
            ("namespace".into(), "default".into()),
            ("name".into(), "settings".into()),
            ("scope".into(), scope.into()),
            ("repository".into(), "owner/repo".into()),
            ("commit".into(), "a".repeat(40)),
            ("url".into(), "https://tempo.example/api/search".into()),
            (
                "cluster_attribute".into(),
                "resource.k8s.cluster.name".into(),
            ),
            ("start_unix".into(), "1".into()),
            ("end_unix".into(), "2".into()),
        ]);
        parameters.retain(|key, _| descriptor.parameters.contains(key));
        let binding = descriptor
            .binding(
                &parameters,
                &BTreeMap::from([
                    ("executable".into(), "/operator/tool".into()),
                    ("supervisor".into(), "/operator/supervisor".into()),
                ]),
            )
            .unwrap();
        let config = Config::parse(binding.argv.iter().skip(1).cloned().collect()).unwrap();
        assert_eq!(config.version().name(), "2");
        let mut input = json!({});
        for (key, property) in descriptor.input_schema["properties"].as_object().unwrap() {
            input[key] = if property["type"] == "integer" {
                json!(parameters[key].parse::<u64>().unwrap())
            } else {
                json!(parameters[key])
            };
        }
        let schema = jsonschema::validator_for(&descriptor.input_schema).unwrap();
        schema.validate(&input).unwrap();
        if descriptor.id == "kubernetes_cluster" {
            let mut widened = input.clone();
            widened["namespace"] = json!("default");
            assert!(!schema.is_valid(&widened));
        }
        if descriptor.id == "kubernetes" {
            let mut widened = input.clone();
            widened["scope"] = json!("cluster");
            assert!(!schema.is_valid(&widened));
        }
        ToolRegistry::from_toml(
            &CommandRegistry {
                tools: BTreeMap::from([("observed".into(), binding)]),
            }
            .to_toml()
            .unwrap(),
        )
        .unwrap();
    }
}
