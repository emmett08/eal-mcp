//! Independently authored admission cases for typed provider selections.
use eal_collectors::config::*;
use std::{path::PathBuf, time::Duration};
fn command() -> CommandConfig {
    CommandConfig {
        executable: PathBuf::from("/operator/tool"),
        supervisor: PathBuf::from("/operator/supervisor"),
        inherit_env: vec!["PATH".into()],
    }
}
fn github() -> GithubConfig {
    GithubConfig {
        command: command(),
        repository: "owner/repo".into(),
        commit: "a".repeat(40),
    }
}
fn tempo() -> TempoConfig {
    TempoConfig {
        inventory: "/operator/inventory.json".into(),
        url: "https://tempo.example/api/search".parse().unwrap(),
        kind: "Application".into(),
        name: "service".into(),
        cluster_attribute: "resource.k8s.cluster.name".into(),
        kind_attribute: "span.dpe.rollout_kind".into(),
        name_attribute: "span.dpe.rollout_name".into(),
        token_env: None,
        tenant_env: None,
        max_window: Duration::from_secs(86400),
        limit: 20,
    }
}
fn admit(selected: ProviderConfig) -> Result<Config, String> {
    Config::new(selected, CollectorLimits::default(), ContractVersion::V2)
}
#[test]
fn direct_construction_and_cli_use_one_invariant_boundary() {
    assert!(admit(ProviderConfig::Github(github())).is_ok());
    assert!(admit(ProviderConfig::Tempo(tempo())).is_ok());
    for path in ["relative", "", "a/b"] {
        let mut c = github();
        c.command.executable = path.into();
        assert!(admit(ProviderConfig::Github(c)).is_err());
    }
    for repository in [
        "owner/..",
        "owner",
        "/repo",
        "owner/repo/extra",
        "-owner/repo",
    ] {
        let mut c = github();
        c.repository = repository.into();
        assert!(admit(ProviderConfig::Github(c)).is_err());
    }
    for commit in ["main".into(), "g".repeat(40), "a".repeat(41)] {
        let mut c = github();
        c.commit = commit;
        assert!(admit(ProviderConfig::Github(c)).is_err());
    }
    for env in [
        vec!["TOKEN".into(), "TOKEN".into()],
        vec!["BAD.NAME".into()],
        vec!["A".repeat(1025)],
    ] {
        let mut c = github();
        c.command.inherit_env = env;
        assert!(admit(ProviderConfig::Github(c)).is_err());
    }
    for url in [
        "http://remote.example/api/search",
        "https://user:secret@example/api/search",
        "https://example/api/search?token=secret",
        "https://example/api/search#secret",
        "file:///tmp/traces",
    ] {
        let mut c = tempo();
        c.url = url.parse().unwrap();
        let error = admit(ProviderConfig::Tempo(c)).unwrap_err();
        assert!(!error.contains("secret"));
    }
    for limit in [0, 1001, usize::MAX] {
        let mut c = tempo();
        c.limit = limit;
        assert!(admit(ProviderConfig::Tempo(c)).is_err());
    }
    for window in [Duration::ZERO, Duration::from_secs(2_592_001)] {
        let mut c = tempo();
        c.max_window = window;
        assert!(admit(ProviderConfig::Tempo(c)).is_err());
    }
    let mut c = tempo();
    c.token_env = Some("TOKEN.BAD".into());
    assert!(admit(ProviderConfig::Tempo(c)).is_err());
    for seconds in [0, 3601] {
        assert!(CollectorLimits::new(Duration::from_secs(seconds), 1024).is_err());
    }
    for bytes in [0, 255, 16_777_217] {
        assert!(CollectorLimits::new(Duration::from_secs(1), bytes).is_err());
    }
    assert!(CollectorLimits::new(Duration::from_secs(3600), 16_777_216).is_ok());
}
#[test]
fn wrong_provider_settings_are_rejected_before_any_acquisition() {
    let base = [
        "github",
        "--executable",
        "/operator/gh",
        "--supervisor",
        "/operator/supervisor",
        "--repository",
        "owner/repo",
        "--commit",
    ];
    for key in ["--inventory", "--url", "--scope", "--namespace", "--limit"] {
        let mut args = base.map(str::to_owned).to_vec();
        args.extend(["a".repeat(40), key.into(), "unexpected".into()]);
        assert!(Config::parse(args).is_err());
    }
    let mut args = base.map(str::to_owned).to_vec();
    args.push("a".repeat(40));
    let c = Config::parse(args).unwrap();
    assert_eq!(c.limits().timeout(), Duration::from_secs(30));
    assert_eq!(c.limits().max_bytes(), 1_048_576);
    assert_eq!(c.version(), ContractVersion::V1);
}
#[test]
fn scope_requires_explicit_selection_and_never_widens_from_missing_namespace() {
    let base = KubernetesConfig {
        command: command(),
        inventory: "/operator/inventory".into(),
        resource: "nodes".into(),
        kind: "Node".into(),
        name: "worker".into(),
        scope: ResourceScope::Cluster,
    };
    assert!(
        Config::new(
            ProviderConfig::Kubernetes(base.clone()),
            CollectorLimits::default(),
            ContractVersion::V1
        )
        .is_err()
    );
    assert!(admit(ProviderConfig::Kubernetes(base)).is_ok());
    let args = [
        "kubernetes",
        "--executable",
        "/operator/tool",
        "--supervisor",
        "/operator/supervisor",
        "--inventory",
        "/operator/inventory",
        "--resource",
        "nodes",
        "--kind",
        "Node",
        "--name",
        "worker",
        "--contract-version",
        "2",
    ];
    assert!(Config::parse(args.map(str::to_owned).to_vec()).is_err());
    let mut explicit = args.map(str::to_owned).to_vec();
    explicit.extend(["--scope".into(), "cluster".into()]);
    assert!(Config::parse(explicit.clone()).is_ok());
    explicit.extend(["--namespace".into(), "default".into()]);
    assert!(Config::parse(explicit).is_err());
}

#[test]
fn tempo_attributes_require_nonempty_scoped_components() {
    for invalid in [
        "span.",
        "resource.",
        "span..name",
        "resource.service.",
        "resource..name",
        "service.name",
    ] {
        for field in 0..3 {
            let mut selected = tempo();
            match field {
                0 => selected.cluster_attribute = invalid.into(),
                1 => selected.kind_attribute = invalid.into(),
                _ => selected.name_attribute = invalid.into(),
            }
            assert!(
                admit(ProviderConfig::Tempo(selected))
                    .unwrap_err()
                    .contains("scoped dotted identifier")
            );
        }
    }
    for valid in [
        "span.name",
        "resource.service.name",
        "span.dpe_rollout.name",
    ] {
        let mut selected = tempo();
        selected.cluster_attribute = valid.into();
        selected.kind_attribute = valid.into();
        selected.name_attribute = valid.into();
        assert!(admit(ProviderConfig::Tempo(selected)).is_ok());
    }
}

#[cfg(unix)]
#[test]
fn non_utf8_operator_paths_are_rejected_without_replacement() {
    use std::{ffi::OsString, os::unix::ffi::OsStringExt};

    let non_utf8 = PathBuf::from(OsString::from_vec(b"/operator/\xff-tool".to_vec()));
    for field in 0..2 {
        let mut selected = github();
        match field {
            0 => selected.command.executable = non_utf8.clone(),
            _ => selected.command.supervisor = non_utf8.clone(),
        }
        assert!(
            admit(ProviderConfig::Github(selected))
                .unwrap_err()
                .contains("UTF-8")
        );
    }
    let selected = KubernetesConfig {
        command: command(),
        inventory: non_utf8.clone(),
        resource: "nodes".into(),
        kind: "Node".into(),
        name: "worker".into(),
        scope: ResourceScope::Cluster,
    };
    assert!(
        admit(ProviderConfig::Kubernetes(selected))
            .unwrap_err()
            .contains("UTF-8")
    );
    let mut selected = tempo();
    selected.inventory = non_utf8;
    assert!(
        admit(ProviderConfig::Tempo(selected))
            .unwrap_err()
            .contains("UTF-8")
    );

    // The actual Unicode replacement character is a valid, distinct selection.
    let path = PathBuf::from("/operator/\u{fffd}-tool");
    let mut selected = github();
    selected.command.executable = path.clone();
    let admitted = admit(ProviderConfig::Github(selected)).unwrap();
    assert_eq!(admitted.selected().command().unwrap().executable, path);
}
