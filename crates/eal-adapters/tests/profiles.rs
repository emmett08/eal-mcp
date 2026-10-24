//! Contract cases for compatibility profiles consumed by authoring and execution.
use eal_adapters::profiles;
use std::collections::BTreeMap;

#[test]
fn profile_argument_admission_matches_the_published_measurement_inputs() {
    for (id, arguments) in [
        ("artifact_json", vec!["inputs"]),
        ("scoped_json", vec!["inputs"]),
        (
            "kubectl_get",
            vec!["context", "namespace", "resource", "name"],
        ),
        ("gh_api_get", vec!["endpoint"]),
        ("http_json_get", vec!["url"]),
    ] {
        let profile = profiles::require(id).unwrap();
        assert_eq!(profile.arguments, arguments);
        let mut values: BTreeMap<_, _> = arguments
            .iter()
            .map(|name| ((*name).to_owned(), "literal".to_owned()))
            .collect();
        assert!(profile.validate_arguments(&values).is_ok());
        values.insert("extra_execution_authority".into(), "/bin/other".into());
        assert!(profile.validate_arguments(&values).is_err());
        values.remove("extra_execution_authority");
        values.remove(arguments[0]);
        assert!(profile.validate_arguments(&values).is_err());
        let view = profile.planning_view();
        assert_eq!(view.as_object().unwrap().len(), 3);
        assert!(view.get("inherit_env").is_none());
        assert!(view.get("fixed_env").is_none());
    }
    assert!(profiles::get("arbitrary_executable").is_none());
}

#[test]
fn github_execution_policy_keeps_prompting_disabled_without_ambient_inheritance() {
    let environment = profiles::GITHUB.environment();
    assert_eq!(
        environment.get("GH_PROMPT_DISABLED").map(String::as_str),
        Some("1")
    );
    assert_eq!(environment.get("GH_PAGER").map(String::as_str), Some("cat"));
    assert!(environment.keys().all(|name| {
        profiles::GITHUB.inherit_env.contains(&name.as_str())
            || profiles::GITHUB
                .fixed_env
                .iter()
                .any(|(fixed, _)| *fixed == name)
    }));
    assert!(!profiles::KUBECTL.inherit_env.contains(&"GH_TOKEN"));
}
