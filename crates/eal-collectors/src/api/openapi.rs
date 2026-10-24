//! A bounded JSON OpenAPI subset compiled into one approved measurement contract.
mod schema;
use super::config::pinned;
use eal_adapters::http::JsonClient;
use reqwest::Method;
use schema::resolve;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

pub(super) struct Operation {
    operation_id: String,
    specification_sha256: String,
    path: String,
    method: Method,
    input: Value,
    responses: BTreeMap<String, Value>,
}
impl Operation {
    pub fn load(path: &Path, digest: &str, operation_id: &str) -> Result<Self, String> {
        let spec = eal_adapters::parse_json(&pinned(path, digest)?)
            .map_err(|_| "OpenAPI specification requires bounded strict JSON")?;
        if spec["openapi"]
            .as_str()
            .is_none_or(|version| !version.starts_with("3.0.") && !version.starts_with("3.1."))
        {
            return Err("OpenAPI JSON import supports explicit 3.0 and 3.1 descriptions".into());
        }
        let paths = spec["paths"]
            .as_object()
            .ok_or("OpenAPI specification lacks paths")?;
        let mut selected = None;
        let mut ids = BTreeSet::new();
        for (path, item) in paths {
            let item = resolve(item, &spec, &mut BTreeSet::new(), 0)?;
            for verb in [
                "get", "post", "put", "patch", "delete", "head", "options", "trace",
            ] {
                let Some(operation) = item.get(verb) else {
                    continue;
                };
                let Some(id) = operation["operationId"].as_str() else {
                    continue;
                };
                if !ids.insert(id.to_owned()) {
                    return Err("OpenAPI repeats an operationId".into());
                }
                if id == operation_id {
                    if !matches!(verb, "get" | "post") {
                        return Err(
                            "OpenAPI evidence supports approved GET and POST operations".into()
                        );
                    }
                    selected = Some((
                        path.clone(),
                        verb.to_owned(),
                        item.clone(),
                        operation.clone(),
                    ));
                }
            }
        }
        let (path, verb, item, operation) =
            selected.ok_or("Selected OpenAPI operationId is unavailable")?;
        if !path.starts_with('/') || path.starts_with("//") || path.contains(['?', '#', '\\']) {
            return Err("Invalid OpenAPI operation path".into());
        }
        let mut parameters = BTreeMap::new();
        for declaration in [&item, &operation] {
            let mut local = BTreeSet::new();
            if let Some(value) = declaration.get("parameters") {
                for parameter in value
                    .as_array()
                    .ok_or("OpenAPI parameters require an array")?
                {
                    let parameter = resolve(parameter, &spec, &mut BTreeSet::new(), 0)?;
                    let name = parameter["name"]
                        .as_str()
                        .filter(|name| !name.is_empty())
                        .ok_or("OpenAPI parameter requires a name")?
                        .to_owned();
                    let location = parameter["in"]
                        .as_str()
                        .ok_or("OpenAPI parameter requires a location")?
                        .to_owned();
                    if !matches!(location.as_str(), "path" | "query")
                        || !local.insert((location.clone(), name.clone()))
                    {
                        return Err(
                            "OpenAPI import requires distinct path and query parameters".into()
                        );
                    }
                    if parameter.get("style").is_some_and(|style| {
                        style != if location == "path" { "simple" } else { "form" }
                    }) || parameter["allowReserved"] == true
                    {
                        return Err("Unsupported OpenAPI parameter serialization".into());
                    }
                    let schema = parameter
                        .get("schema")
                        .ok_or("OpenAPI parameter requires a schema")?;
                    if !matches!(
                        schema["type"].as_str(),
                        Some("string" | "integer" | "number" | "boolean")
                    ) {
                        return Err(
                            "OpenAPI path and query parameters require primitive schemas".into(),
                        );
                    }
                    parameters.insert((location, name), parameter);
                }
            }
        }
        let mut properties = serde_json::Map::new();
        let mut required = Vec::new();
        for location in ["path", "query"] {
            let mut fields = serde_json::Map::new();
            let mut needed = Vec::new();
            for ((at, name), parameter) in &parameters {
                if at != location {
                    continue;
                }
                if parameter["required"] == true {
                    needed.push(name.clone());
                }
                if location == "path"
                    && (parameter["required"] != true
                        || !path.split('/').any(|part| part == format!("{{{name}}}")))
                {
                    return Err(
                        "OpenAPI path parameters must occupy required complete segments".into(),
                    );
                }
                fields.insert(name.clone(), parameter["schema"].clone());
            }
            if !fields.is_empty() {
                if !needed.is_empty() {
                    required.push(location.to_owned());
                }
                properties.insert(location.into(),json!({"type":"object","additionalProperties":false,"properties":fields,"required":needed}));
            }
        }
        for segment in path.split('/') {
            if segment.contains(['{', '}']) {
                let name = segment
                    .strip_prefix('{')
                    .and_then(|s| s.strip_suffix('}'))
                    .ok_or("OpenAPI parameter must occupy one path segment")?;
                if !parameters.contains_key(&("path".into(), name.into())) {
                    return Err("OpenAPI path has an undeclared parameter".into());
                }
            } else if matches!(segment, "." | "..") {
                return Err("OpenAPI path cannot traverse parent segments".into());
            }
        }
        if let Some(body) = operation.get("requestBody") {
            if verb != "post" {
                return Err("OpenAPI GET request bodies are unsupported".into());
            }
            let body = resolve(body, &spec, &mut BTreeSet::new(), 0)?;
            let schema = body
                .pointer("/content/application~1json/schema")
                .ok_or("OpenAPI request body requires application/json schema")?
                .clone();
            if body["required"] == true {
                required.push("body".into());
            }
            properties.insert("body".into(), schema);
        }
        let mut responses = BTreeMap::new();
        for (status, response) in operation["responses"]
            .as_object()
            .ok_or("OpenAPI operation requires responses")?
        {
            if !status.starts_with('2')
                || !(status == "2XX"
                    || status.len() == 3 && status.bytes().all(|b| b.is_ascii_digit()))
            {
                continue;
            }
            let response = resolve(response, &spec, &mut BTreeSet::new(), 0)?;
            let schema = response
                .pointer("/content/application~1json/schema")
                .ok_or("OpenAPI successful responses require application/json schemas")?
                .clone();
            validate_schema(&schema)?;
            responses.insert(status.clone(), schema);
        }
        if responses.is_empty() {
            return Err("OpenAPI operation has no supported successful JSON response".into());
        }
        let input = json!({"type":"object","additionalProperties":false,"properties":properties,"required":required});
        validate_schema(&input)?;
        Ok(Self {
            operation_id: operation_id.into(),
            specification_sha256: digest.into(),
            path,
            method: if verb == "get" {
                Method::GET
            } else {
                Method::POST
            },
            input,
            responses,
        })
    }
    pub fn collector_id(&self) -> String {
        let hash = Sha256::digest(
            format!("{}\n{}", self.specification_sha256, self.operation_id).as_bytes(),
        );
        format!(
            "openapi.{}",
            hash[..12]
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<String>()
        )
    }
    pub fn input_schema(&self) -> Value {
        self.input.clone()
    }
    pub fn response_schema(&self) -> Value {
        json!({"anyOf":self.responses.values().collect::<Vec<_>>()})
    }
    pub async fn collect(&self, client: &mut JsonClient, input: &Value) -> Result<Value, String> {
        let mut url = client.url("/").map_err(|e| e.to_string())?;
        {
            let mut segments = url
                .path_segments_mut()
                .map_err(|_| "OpenAPI origin cannot hold path segments")?;
            segments.clear();
            for segment in self.path.strip_prefix('/').unwrap_or(&self.path).split('/') {
                let text = if let Some(name) =
                    segment.strip_prefix('{').and_then(|s| s.strip_suffix('}'))
                {
                    scalar(&input["path"][name])?
                } else {
                    segment.into()
                };
                if matches!(text.as_str(), "." | "..") {
                    return Err("OpenAPI path input cannot traverse parent segments".into());
                }
                segments.push(&text);
            }
        }
        if let Some(query) = input.get("query").and_then(Value::as_object) {
            for (name, value) in query {
                url.query_pairs_mut().append_pair(name, &scalar(value)?);
            }
        }
        let response = client
            .request(self.method.clone(), url, input.get("body"))
            .await
            .map_err(|e| e.to_string())?;
        let status = client
            .status()
            .ok_or("OpenAPI response lacks HTTP status")?;
        let schema = self
            .responses
            .get(&status.to_string())
            .or_else(|| self.responses.get("2XX"))
            .ok_or("OpenAPI returned an undeclared successful status")?;
        jsonschema::validator_for(schema)
            .map_err(|_| "Invalid OpenAPI response schema")?
            .validate(&response)
            .map_err(|_| "OpenAPI response differs from its declared schema")?;
        Ok(
            json!({"collected":true,"operation_id":self.operation_id,"specification_sha256":self.specification_sha256,"http_status":status,"response_schema_validated":true,"coverage":"unspecified","observation":response}),
        )
    }
}
fn scalar(value: &Value) -> Result<String, String> {
    match value {
        Value::String(text) => Ok(text.clone()),
        Value::Bool(_) | Value::Number(_) => Ok(value.to_string()),
        _ => Err("OpenAPI parameter requires a primitive value".into()),
    }
}
fn validate_schema(schema: &Value) -> Result<(), String> {
    jsonschema::validator_for(schema)
        .map(|_| ())
        .map_err(|_| "Unsupported or invalid OpenAPI JSON schema".into())
}
