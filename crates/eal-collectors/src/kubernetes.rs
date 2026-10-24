//! Named resource observations across one declared production inventory.
use serde_json::{Value, json};

pub(crate) fn collect(
    config: &crate::Config,
    request: &crate::Request,
    started: std::time::Instant,
) -> Result<Value, String> {
    request.fields(&["resource", "namespace", "name"])?;
    let resource = config.get("--resource")?;
    let namespace = config.get("--namespace")?;
    let name = config.get("--name")?;
    let kind = config.get("--kind")?;
    for (key, value) in [
        ("resource", resource),
        ("namespace", namespace),
        ("name", name),
    ] {
        request.matches(key, value)?;
    }
    let (inventory, digest) = crate::inventory::read(config)?;
    let mut clusters = Vec::new();
    let mut all_present = true;
    let mut spent = 0usize;
    for cluster in &inventory.clusters {
        let output = crate::process::run(
            config,
            vec![
                "get".into(),
                resource.into(),
                name.into(),
                "--namespace".into(),
                namespace.into(),
                "--context".into(),
                cluster.context.clone(),
                "--ignore-not-found".into(),
                "--output".into(),
                "json".into(),
                format!("--request-timeout={}s", config.timeout.as_secs()),
            ],
            started,
        )?;
        spent = spent.saturating_add(output.len());
        if spent > config.max_bytes {
            return Err("Kubernetes responses exceed the aggregate byte allowance".into());
        }
        let present = !output.iter().all(u8::is_ascii_whitespace);
        if present {
            let object = crate::process::object(&output)?;
            if object["metadata"]["name"] != name
                || object["metadata"]["namespace"] != namespace
                || object["kind"] != kind
            {
                return Err("Kubernetes returned a different resource identity".into());
            }
            let api = object["apiVersion"]
                .as_str()
                .ok_or("Resource lacks apiVersion")?;
            let expected_group = resource
                .split_once('.')
                .map(|(_, group)| group)
                .unwrap_or("");
            let group = api.split_once('/').map(|(group, _)| group).unwrap_or("");
            if group != expected_group {
                return Err("Kubernetes returned a different API group".into());
            }
            for key in ["uid", "resourceVersion"] {
                if object["metadata"][key].as_str().is_none_or(str::is_empty) {
                    return Err("Resource lacks UID or resourceVersion".into());
                }
            }
            if !object["metadata"]["generation"].is_u64() {
                return Err("Resource lacks an integer generation".into());
            }
            clusters.push(json!({"cluster":cluster.id,"context":cluster.context,"present":true,"resource":{"apiVersion":api,"kind":kind,"metadata":{"name":name,"namespace":namespace,"uid":object["metadata"]["uid"],"resourceVersion":object["metadata"]["resourceVersion"],"generation":object["metadata"]["generation"]},"status":object.get("status")}}));
        } else {
            all_present = false;
            clusters.push(json!({"cluster":cluster.id,"context":cluster.context,"present":false}));
        }
    }
    Ok(
        json!({"inventory_revision":inventory.revision,"inventory_sha256":digest,"cluster_count":inventory.clusters.len(),"all_present":all_present,"clusters":clusters}),
    )
}
