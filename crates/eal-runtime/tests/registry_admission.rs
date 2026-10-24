//! The actual runtime consumes the same independent corpus as contract clients.
use eal_runtime::acquisition::ToolRegistry;
use serde_json::Value;
#[test]
fn runtime_registry_matches_independent_contract_corpus() {
    for case in
        serde_json::from_str::<Vec<Value>>(eal_collector_contracts::REGISTRY_CORPUS).unwrap()
    {
        assert_eq!(
            ToolRegistry::from_toml(case["registry"].as_str().unwrap()).is_ok(),
            case["accepted"].as_bool().unwrap(),
            "{}",
            case["name"]
        );
    }
}
#[test]
fn malformed_registry_diagnostics_never_echo_values_or_source_excerpts() {
    let secret = "synthetic-review-secret";
    for suffix in [
        format!("env={{LAB_API_TOKEN=\"{secret}\", BROKEN=}}"),
        format!("env={{LAB_API_TOKEN=\"{secret}"),
        format!("env={{LAB_API_TOKEN=\"\"\"{secret}\n"),
        format!("env={{LAB_API_TOKEN=\"{secret}\",LAB_API_TOKEN=\"{secret}\"}}"),
        format!("\"κλειδί\"=\"{secret}\"\ninvalid="),
        format!("env={{LAB_API_TOKEN=42}}\n\"{secret}\"=true"),
    ] {
        let text = format!(
            "[tools.reader]\nkind=\"command\"\nversion=\"1\"\nargv=[\"reader\"]\n{suffix}\n"
        );
        let error = ToolRegistry::from_toml(&text).unwrap_err().to_string();
        assert!(error.starts_with("EAL_"), "{error}");
        assert!(!error.contains(secret), "{error}");
        assert!(!error.contains("LAB_API_TOKEN"), "{error}");
        assert!(!error.contains(&suffix), "{error}");
    }
}
#[test]
fn safe_diagnostics_distinguish_syntax_fields_shape_and_limits() {
    for (source, code) in [
        ("[tools.reader]\nkind=", "EAL_REGISTRY_SYNTAX"),
        (
            "[tools.reader]\nkind='command'\nversion='1'\nargv=['reader']\nunknown=true",
            "EAL_REGISTRY_FIELDS",
        ),
        (
            "[tools.reader]\nkind='command'\nversion='1'\nargv=42",
            "EAL_REGISTRY_SHAPE",
        ),
        (
            "[tools.reader]\nkind='command'\nversion='1'\nargv=['reader']\ntimeout_seconds=0",
            "EAL_BINDING_LIMIT",
        ),
    ] {
        assert!(
            ToolRegistry::from_toml(source)
                .unwrap_err()
                .message
                .starts_with(code)
        );
    }
}
