//! Closed task and trusted single-host configuration contracts.
use eal_core::{ExecutionLimits, canonical, expressions::numbers};
use serde_json::{Map, Value, json};
use std::{
    collections::BTreeSet,
    path::{Component, Path, PathBuf},
};

pub const TASK_SCHEMA: &str = "EAL/MAS-coordination-task/3";
pub const FINDING_SCHEMA: &str = "EAL/MAS-coordination-finding/3";
pub const MAX_CLAIMS: usize = 16;

fn check_text(value: &str, name: &str, maximum: usize) -> Result<(), String> {
    let whitespace =
        |character: char| character.is_whitespace() || ('\u{1c}'..='\u{1f}').contains(&character);
    if value.is_empty() || value.trim_matches(whitespace) != value || value.len() > maximum {
        return Err(format!(
            "{name} requires nonempty bounded text without surrounding whitespace"
        ));
    }
    Ok(())
}
pub fn text<'a>(value: &'a Value, name: &str, maximum: usize) -> Result<&'a str, String> {
    let value = value.as_str().ok_or_else(|| {
        format!("{name} requires nonempty bounded text without surrounding whitespace")
    })?;
    check_text(value, name, maximum)?;
    Ok(value)
}

/// Finite JSON with the source's 32-level/4096-node and Python spaced-JSON byte bounds.
pub fn json_object(value: &Value, name: &str, maximum: usize) -> Result<Value, String> {
    if !value.is_object() {
        return Err(format!("{name} requires a JSON object"));
    }
    fn visit(
        value: &Value,
        depth: usize,
        nodes: &mut usize,
        spaces: &mut usize,
        name: &str,
    ) -> Result<(), String> {
        *nodes += 1;
        if depth > 32 || *nodes > 4096 {
            return Err(format!("{name} exceeds structural bounds"));
        }
        match value {
            Value::Object(values) => {
                *spaces = spaces
                    .saturating_add(values.len())
                    .saturating_add(values.len().saturating_sub(1));
                for value in values.values() {
                    visit(value, depth + 1, nodes, spaces, name)?;
                }
            }
            Value::Array(values) => {
                *spaces = spaces.saturating_add(values.len().saturating_sub(1));
                for value in values {
                    visit(value, depth + 1, nodes, spaces, name)?;
                }
            }
            Value::Number(number) if number.as_str().contains(['.', 'e', 'E']) => {
                if number.as_f64().is_none_or(|value| !value.is_finite()) {
                    return Err(format!("{name} requires finite JSON values"));
                }
            }
            _ => {}
        }
        Ok(())
    }
    let mut spaces = 0usize;
    visit(value, 0, &mut 0, &mut spaces, name)?;
    let limits = ExecutionLimits {
        record_bytes: maximum.max(1),
        json_depth: 32,
        json_nodes: 4096,
        ..Default::default()
    };
    let encoded = canonical::canonical_json_bytes(value, &limits)
        .map_err(|_| format!("{name} exceeds its byte bound"))?;
    if encoded.len().saturating_add(spaces) > maximum {
        return Err(format!("{name} exceeds its byte bound"));
    }
    fn copy(value: &Value) -> Value {
        match value {
            Value::Object(values) => Value::Object(
                values
                    .iter()
                    .map(|(key, value)| (key.clone(), copy(value)))
                    .collect(),
            ),
            Value::Array(values) => Value::Array(values.iter().map(copy).collect()),
            Value::Number(number) if number.as_str().contains(['.', 'e', 'E']) => Value::Number(
                serde_json::Number::from_f64(number.as_f64().expect("finite number preflight"))
                    .expect("finite number preflight"),
            ),
            Value::Number(number) if number.as_str() == "-0" => json!(0),
            _ => value.clone(),
        }
    }
    Ok(copy(value))
}

fn closed<'a>(
    value: &'a Value,
    fields: &[&str],
    message: &str,
) -> Result<&'a Map<String, Value>, String> {
    value
        .as_object()
        .filter(|object| {
            object.len() == fields.len() && fields.iter().all(|name| object.contains_key(*name))
        })
        .ok_or_else(|| message.into())
}

#[derive(Clone, Debug, PartialEq)]
pub struct ServerSpec {
    pub workspace: PathBuf,
    pub registry: Option<PathBuf>,
    pub database: Option<PathBuf>,
    pub methods: Option<String>,
    pub limits: Option<PathBuf>,
}
impl ServerSpec {
    pub fn from_value(value: &Value, base: &Path) -> Result<Self, String> {
        let fields = closed(
            value,
            &["workspace", "registry", "database", "methods", "limits"],
            "Server specification requires exactly its declared fields",
        )?;
        if fields
            .values()
            .any(|value| !value.is_null() && !value.is_string())
        {
            return Err("Server JSON paths and method factory require strings".into());
        }
        if !base.is_absolute() {
            return Err("Server path base must be absolute".into());
        }
        let resolve = |name: &str| -> Result<Option<PathBuf>, String> {
            if fields[name].is_null() && name != "workspace" {
                return Ok(None);
            }
            let value = fields[name]
                .as_str()
                .ok_or_else(|| format!("{name} requires a filesystem path"))?;
            check_text(value, name, 4096)?;
            Ok(Some(resolve_path(base, value)?))
        };
        let workspace = resolve("workspace")?.ok_or("workspace requires a filesystem path")?;
        let registry = resolve("registry")?;
        let database = resolve("database")?.or_else(|| Some(workspace.join(".eal/runs.sqlite3")));
        let limits = resolve("limits")?;
        let result = Self {
            workspace,
            registry,
            database,
            methods: fields["methods"].as_str().map(str::to_owned),
            limits,
        };
        result.validate()?;
        Ok(result)
    }
    pub fn validate(&self) -> Result<(), String> {
        for path in std::iter::once(&self.workspace)
            .chain(self.registry.iter())
            .chain(self.database.iter())
            .chain(self.limits.iter())
        {
            if !path.is_absolute()
                || path.to_str().is_none()
                || path.as_os_str().as_encoded_bytes().contains(&0)
            {
                return Err("Server paths must be absolute UTF-8 filesystem paths".into());
            }
        }
        if self
            .methods
            .as_deref()
            .is_some_and(|method| !["builtin", "engineering", "aspic", "all"].contains(&method))
        {
            return Err("methods must be builtin, engineering, aspic or all; Python factories are not loaded".into());
        }
        Ok(())
    }
    pub fn to_value(&self) -> Result<Value, String> {
        self.validate()?;
        Ok(
            json!({"workspace":self.workspace,"registry":self.registry,"database":self.database,"methods":self.methods,"limits":self.limits}),
        )
    }
}
fn resolve_path(base: &Path, value: &str) -> Result<PathBuf, String> {
    if value.contains('\0') {
        return Err("embedded null byte".into());
    }
    let expanded = if value == "~" || value.starts_with("~/") {
        let home = std::env::var_os("HOME").ok_or("Cannot expand home directory without HOME")?;
        PathBuf::from(home).join(value.strip_prefix("~/").unwrap_or(""))
    } else {
        PathBuf::from(value)
    };
    let absolute = if expanded.is_absolute() {
        expanded
    } else {
        base.join(expanded)
    };
    let mut result = PathBuf::new();
    for component in absolute.components() {
        match component {
            Component::ParentDir => {
                result.pop();
            }
            Component::CurDir => {}
            other => {
                result.push(other.as_os_str());
                if let Ok(canonical) = result.canonicalize() {
                    result = canonical;
                }
            }
        }
    }
    Ok(result)
}

#[derive(Clone, Debug, PartialEq)]
pub struct TaskSpec {
    pub task_id: String,
    pub question: String,
    pub entry_id: String,
    pub claims: Vec<String>,
    pub profile: String,
    pub requirements: Value,
}
impl Drop for TaskSpec {
    fn drop(&mut self) {
        discard(std::mem::take(&mut self.requirements));
    }
}

fn discard(value: Value) {
    let mut pending = vec![value];
    while let Some(value) = pending.pop() {
        match value {
            Value::Array(values) => pending.extend(values),
            Value::Object(values) => pending.extend(values.into_values()),
            _ => {}
        }
    }
}
impl TaskSpec {
    pub fn from_value(value: &Value) -> Result<Self, String> {
        let fields = closed(
            value,
            &[
                "task_id",
                "question",
                "entry_id",
                "claims",
                "profile",
                "requirements",
            ],
            "Task specification requires exactly its declared fields",
        )?;
        let claims = fields["claims"]
            .as_array()
            .ok_or("Task JSON claims require an array")?;
        let mut strings = Vec::new();
        for (name, maximum) in [
            ("task_id", 128),
            ("question", 8192),
            ("entry_id", 256),
            ("profile", 128),
        ] {
            strings.push(text(&fields[name], name, maximum)?.to_owned());
        }
        if claims.is_empty()
            || claims.len() > MAX_CLAIMS
            || claims.iter().any(|value| !value.is_string())
            || claims
                .iter()
                .filter_map(Value::as_str)
                .collect::<BTreeSet<_>>()
                .len()
                != claims.len()
        {
            return Err(format!("claims require one to {MAX_CLAIMS} unique strings"));
        }
        let claims = claims
            .iter()
            .map(|claim| text(claim, "claim", 128).map(str::to_owned))
            .collect::<Result<Vec<_>, _>>()?;
        let requirements = json_object(&fields["requirements"], "requirements", 65536)?;
        let [task_id, question, entry_id, profile]: [String; 4] =
            strings.try_into().map_err(|_| "Invalid task fields")?;
        let result = Self {
            task_id,
            question,
            entry_id,
            claims,
            profile,
            requirements,
        };
        result.validate()?;
        Ok(result)
    }
    pub fn validate(&self) -> Result<(), String> {
        for (name, value, maximum) in [
            ("task_id", &self.task_id, 128),
            ("question", &self.question, 8192),
            ("entry_id", &self.entry_id, 256),
            ("profile", &self.profile, 128),
        ] {
            check_text(value, name, maximum)?;
        }
        if self.claims.is_empty()
            || self.claims.len() > MAX_CLAIMS
            || self.claims.iter().collect::<BTreeSet<_>>().len() != self.claims.len()
        {
            return Err(format!("claims require one to {MAX_CLAIMS} unique strings"));
        }
        for claim in &self.claims {
            check_text(claim, "claim", 128)?;
        }
        let requirements = json_object(&self.requirements, "requirements", 65536)?;
        crate::profiles::validate_requirements(&self.profile, &self.claims, &requirements)
    }
    pub fn to_value(&self) -> Result<Value, String> {
        self.validate()?;
        Ok(
            json!({"task_id":self.task_id,"question":self.question,"entry_id":self.entry_id,"claims":self.claims,"profile":self.profile,"requirements":self.requirements}),
        )
    }
}

pub fn validate_anchor(anchor: &Value) -> Result<TaskSpec, String> {
    let fields = closed(
        anchor,
        &[
            "schema",
            "task_digest",
            "task",
            "source_digest",
            "method_registry_fingerprint",
            "registered_context",
            "pinned_at",
        ],
        "Task anchor requires the current closed schema",
    )?;
    if fields["schema"] != TASK_SCHEMA {
        return Err("Task anchor requires the current closed schema".into());
    }
    let task = TaskSpec::from_value(&fields["task"])?;
    json_object(&fields["registered_context"], "registered_context", 16384)?;
    for name in [
        "task_digest",
        "source_digest",
        "method_registry_fingerprint",
    ] {
        if fields[name].as_str().is_none_or(|digest| {
            digest.len() != 64
                || !digest
                    .bytes()
                    .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        }) {
            return Err(format!("{name} requires a SHA-256 digest"));
        }
    }
    let instant = fields["pinned_at"]
        .as_str()
        .ok_or("Timestamp must include a timezone")?;
    eal_core::validation::parse_time(instant)?;
    let content = Value::Object(
        fields
            .iter()
            .filter(|(key, _)| *key != "task_digest")
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect(),
    );
    if canonical::canonical_digest(&content, &ExecutionLimits::default())? != fields["task_digest"]
    {
        return Err("Task anchor digest differs from its content".into());
    }
    Ok(task)
}

pub fn make_finding(
    anchor: &Value,
    check: &Value,
    assessments: Option<&Value>,
    packets: Option<&Value>,
    explanations: Option<&Value>,
    calls: Option<&Value>,
) -> Result<Value, String> {
    validate_anchor(anchor)?;
    let limits = ExecutionLimits {
        record_bytes: 64 * 1024 * 1024,
        json_nodes: 64 * 1024 * 1024,
        json_depth: 128,
        ..Default::default()
    };
    // Bound borrowed, directly constructed JSON before the output clones it.
    // The eventual digest applies this same finite JSON contract.
    for value in std::iter::once(check)
        .chain(assessments)
        .chain(packets)
        .chain(explanations)
        .chain(calls)
    {
        canonical::canonical_json_bytes(value, &limits)?;
    }
    let empty = json!({});
    let empty_calls = json!([]);
    let assessments = assessments
        .filter(|value| !value.is_null())
        .unwrap_or(&empty);
    let rows = assessments
        .as_object()
        .ok_or("Finding assessments require an object")?;
    let mut collected = json!(0);
    let mut reused = json!(0);
    for row in rows.values() {
        collected = numbers::arithmetic(
            "+",
            &collected,
            row.get("collected_count")
                .ok_or("Assessment lacks collected_count")?,
        )?;
        reused = numbers::arithmetic(
            "+",
            &reused,
            row.get("reused_count")
                .ok_or("Assessment lacks reused_count")?,
        )?;
    }
    let mut value = json!({"schema":FINDING_SCHEMA,"task_digest":anchor["task_digest"],"check":check,"assessments":assessments,"packets":packets.filter(|value| !value.is_null()).unwrap_or(&empty),"explanations":explanations.filter(|value| !value.is_null()).unwrap_or(&empty),"mcp":{"transport":"actual_mcp_stdio","calls":calls.filter(|value| !value.is_null()).unwrap_or(&empty_calls)},"collected_count":collected,"reused_count":reused});
    value["finding_id"] = json!(format!(
        "finding_{}",
        canonical::canonical_digest(&value, &limits)?
    ));
    Ok(value)
}
