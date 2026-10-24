//! Identity-checked named resources under explicit operator-selected Kubernetes scope.
use crate::{
    config::{ContractVersion, KubernetesConfig, ResourceScope},
    timing::Timeline,
};
use serde_json::{Value, json};

pub(crate) fn collect(
    config: &crate::Config,
    selected: &KubernetesConfig,
    request: &crate::Request,
    started: std::time::Instant,
    timeline: &mut Timeline<'_>,
) -> Result<Value, String> {
    let resource = selected.resource.as_str();
    let name = selected.name.as_str();
    let kind = selected.kind.as_str();
    match (&selected.scope, config.version()) {
        (ResourceScope::Namespaced(namespace), ContractVersion::V1) => {
            request.fields(&["resource", "namespace", "name"])?;
            request.matches("namespace", namespace)?;
        }
        (ResourceScope::Namespaced(namespace), ContractVersion::V2) => {
            request.fields(&["resource", "scope", "namespace", "name"])?;
            request.matches("scope", "namespaced")?;
            request.matches("namespace", namespace)?;
        }
        (ResourceScope::Cluster, ContractVersion::V2) => {
            request.fields(&["resource", "scope", "name"])?;
            request.matches("scope", "cluster")?;
        }
        _ => return Err("Unsupported Kubernetes contract and scope".into()),
    }
    request.matches("resource", resource)?;
    request.matches("name", name)?;
    let (inventory, digest) = crate::inventory::read(config)?;
    let mut clusters = Vec::new();
    let mut all_present = true;
    let mut spent = 0usize;
    for cluster in &inventory.clusters {
        let item_start = timeline.mark();
        let mut args = vec!["get".into(), resource.into(), name.into()];
        if let ResourceScope::Namespaced(namespace) = &selected.scope {
            args.extend(["--namespace".into(), namespace.clone()]);
        }
        args.extend([
            "--context".into(),
            cluster.context.clone(),
            "--ignore-not-found".into(),
            "--output".into(),
            "json".into(),
            format!("--request-timeout={}s", config.limits().timeout().as_secs()),
        ]);
        let output = crate::process::run(config, args, started)?;
        let interval = timeline.interval(item_start);
        spent = spent.saturating_add(output.len());
        if spent > config.limits().max_bytes() {
            return Err("Kubernetes responses exceed the aggregate byte allowance".into());
        }
        let present = !output.iter().all(u8::is_ascii_whitespace);
        let mut item = json!({"cluster":cluster.id,"context":cluster.context,"present":present});
        if present {
            let object = crate::process::object(&output)?;
            let metadata = object["metadata"]
                .as_object()
                .ok_or("Resource lacks metadata")?;
            let namespace_matches = match &selected.scope {
                ResourceScope::Namespaced(namespace) => {
                    metadata.get("namespace").and_then(Value::as_str) == Some(namespace.as_str())
                }
                ResourceScope::Cluster => metadata.get("namespace").is_none(),
            };
            if object["metadata"]["name"] != name || !namespace_matches || object["kind"] != kind {
                return Err("Kubernetes returned a different resource identity".into());
            }
            let api = object["apiVersion"]
                .as_str()
                .filter(|v| !v.is_empty())
                .ok_or("Resource lacks apiVersion")?;
            let expected_group = resource
                .split_once('.')
                .map(|(_, group)| group)
                .unwrap_or("");
            let group = api.split_once('/').map(|(group, _)| group).unwrap_or("");
            if group != expected_group || api.split('/').count() > 2 || api.ends_with('/') {
                return Err("Kubernetes returned a different API group".into());
            }
            for key in ["uid", "resourceVersion"] {
                if object["metadata"][key].as_str().is_none_or(str::is_empty) {
                    return Err("Resource lacks UID or resourceVersion".into());
                }
            }
            let generation = metadata.get("generation");
            if generation.is_some_and(|v| {
                !v.is_u64()
                    || (config.version() == ContractVersion::V2
                        && v.as_u64().is_some_and(|n| n > i64::MAX as u64))
            }) || (config.version() == ContractVersion::V1 && generation.is_none())
            {
                return Err(
                    "Resource requires a valid integer generation under the selected contract"
                        .into(),
                );
            }
            let mut kept = json!({"name":name,"uid":object["metadata"]["uid"],"resourceVersion":object["metadata"]["resourceVersion"]});
            if let ResourceScope::Namespaced(namespace) = &selected.scope {
                kept["namespace"] = json!(namespace);
            }
            if let Some(generation) = generation {
                kept["generation"] = generation.clone();
            }
            item["resource"] =
                json!({"apiVersion":api,"kind":kind,"metadata":kept,"status":object.get("status")});
        } else {
            all_present = false;
        }
        if config.version() == ContractVersion::V2 {
            item["acquisition_interval"] = interval;
        }
        clusters.push(item);
    }
    let mut value = json!({"inventory_revision":inventory.revision,"inventory_sha256":digest,"cluster_count":inventory.clusters.len(),"all_present":all_present,"clusters":clusters});
    if config.version() == ContractVersion::V2 {
        value["resource_scope"] = json!(match selected.scope {
            ResourceScope::Namespaced(_) => "namespaced",
            ResourceScope::Cluster => "cluster",
        });
    }
    Ok(value)
}
