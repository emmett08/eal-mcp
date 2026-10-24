//! Measurement schemas and command templates consumed unchanged by authoring clients.
use super::{ApiConfig, ApiSettings, ApiSource};
use eal_collector_contracts::CollectorDescriptor;
use serde_json::json;

pub(super) fn selected(
    config: &ApiConfig,
    settings: &ApiSettings,
) -> Result<CollectorDescriptor, String> {
    let mut descriptor = template(&settings.source)?;
    if let Some(query) = settings.source.query() {
        descriptor.input_schema["properties"]["query"]["const"] = json!(query);
    }
    match &settings.source {
        ApiSource::DatadogMonitor { id } => {
            descriptor.input_schema["properties"]["id"]["const"] = json!(id)
        }
        ApiSource::DatadogSlo { id } => {
            descriptor.input_schema["properties"]["id"]["const"] = json!(id)
        }
        ApiSource::Openapi {
            specification,
            specification_sha256,
            operation_id,
            ..
        } => {
            let operation =
                super::openapi::Operation::load(specification, specification_sha256, operation_id)?;
            descriptor.id = operation.collector_id();
            descriptor.purpose = format!(
                "Acquire the operator-approved OpenAPI operation {operation_id}. Response conformance and collection remain separate from claim sufficiency."
            );
            descriptor.input_schema = operation.input_schema();
            descriptor.parameters = descriptor.input_schema["properties"]
                .as_object()
                .ok_or("Missing OpenAPI inputs")?
                .keys()
                .cloned()
                .collect();
            descriptor.result_schema["properties"]["observation"] = operation.response_schema();
        }
        _ => {}
    }
    for constraint in &mut descriptor.constraints {
        let eal_collector_contracts::InputConstraint::OrderedWindow { max_seconds, .. } =
            constraint;
        *max_seconds = settings.max_window_seconds;
    }
    descriptor.command.argv = vec![
        "{operator.executable}".into(),
        config.provider.name().into(),
        "--contract-version".into(),
        "2".into(),
        "--measurement".into(),
        config.measurement.clone(),
        "--config".into(),
        config.path.to_str().ok_or("API path must be UTF-8")?.into(),
        "--config-sha256".into(),
        config.sha256.clone(),
    ];
    descriptor.command.inherit_env = settings
        .headers_env
        .values()
        .cloned()
        .chain(settings.bearer_env.iter().cloned())
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect();
    descriptor.validate()?;
    Ok(descriptor)
}

pub(super) fn template(source: &ApiSource) -> Result<CollectorDescriptor, String> {
    let mut properties = serde_json::Map::new();
    if source.query().is_some() {
        properties.insert(
            "query".into(),
            json!({"type":"string","minLength":1,"maxLength":8192}),
        );
    }
    match source {
        ApiSource::DatadogMonitor { .. } => {
            properties.insert("id".into(), json!({"type":"integer","minimum":1}));
        }
        ApiSource::DatadogSlo { .. } => {
            properties.insert(
                "id".into(),
                json!({"type":"string","minLength":1,"maxLength":128,"pattern":"^[A-Za-z0-9-]+$"}),
            );
        }
        _ => {}
    }
    let window = !matches!(
        source,
        ApiSource::DatadogMonitor { .. } | ApiSource::Openapi { .. }
    );
    if window {
        for field in ["start_unix", "end_unix"] {
            properties.insert(
                field.into(),
                json!({"type":"integer","minimum":1,"maximum":253402300799u64}),
            );
        }
    }
    let parameters = properties.keys().cloned().collect::<Vec<_>>();
    let id = source.id();
    let value = json!({
        "id":id,"version":"2",
        "purpose":format!("Acquire {id} measurements under pinned operator scope. Retain query windows and source data. Search completeness and negative inference require separate evidence."),
        "parameters":parameters,
        "input_schema":{"type":"object","additionalProperties":false,"required":parameters,"properties":properties},
        "result_schema":{"type":"object","required":["collected","measurement_only","observation","config_sha256"],"properties":{
            "collected":{"type":"boolean","const":true},"measurement_only":{"type":"boolean","const":true},"observation":{},"config_sha256":{"type":"string","pattern":"^[0-9a-f]{64}$"}}},
        "command":{"argv":["{operator.executable}",source.provider().name(),"--contract-version","2","--measurement",id,"--config",format!("{{operator.{id}_config}}"),"--config-sha256",format!("{{operator.{id}_sha256}}")],
            "inherit_env":[],"timeout_seconds":30,"max_output_bytes":1048576,"parallel_safe":true},
        "acquisition_predicate":"collected == true",
        "constraints":if window {json!([{"kind":"ordered_window","start":"start_unix","end":"end_unix","max_seconds":86400}])} else {json!([])}
    });
    serde_json::from_value(value).map_err(|_| "Cannot construct API descriptor".into())
}
