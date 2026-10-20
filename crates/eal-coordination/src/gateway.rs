//! Trusted registered acquisition and independent persisted MCP retrieval.
use crate::{
    contracts::{FINDING_SCHEMA, ServerSpec, TaskSpec, make_finding, validate_anchor},
    host::HostPaths,
    persistent_mcp::PersistentMcp,
    profiles::check_task_with_host,
};
use eal_core::{ExecutionLimits, canonical, validation::parse_time};
use eal_mcp::{host, settings::ServerSettings};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
    time::{Duration, Instant},
};
pub type CallGate = Arc<dyn Fn(&str, &Value, &str) -> Result<(), String> + Send + Sync>;
#[derive(Clone)]
pub struct GatewayOptions {
    pub host_paths: HostPaths,
    pub environment: BTreeMap<String, String>,
    pub operation_timeout: Duration,
}
impl std::fmt::Debug for GatewayOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GatewayOptions")
            .field("host_paths", &self.host_paths)
            .field("operation_timeout", &self.operation_timeout)
            .finish_non_exhaustive()
    }
}
impl GatewayOptions {
    pub fn for_host(host_paths: &HostPaths) -> Result<Self, String> {
        Ok(Self {
            host_paths: host_paths.clone(),
            environment: eal_mcp::application::environment()?,
            operation_timeout: Duration::from_secs(60),
        })
    }
}
pub struct McpGateway {
    server: ServerSpec,
    anchor: Value,
    task: TaskSpec,
    options: GatewayOptions,
    on_call: Option<CallGate>,
    calls: Vec<Value>,
}
const WRAPPER: &[&str] = &[
    "schema",
    "entry_id",
    "claim",
    "source_digest",
    "context_fingerprint",
    "collection_id",
    "assessment_id",
    "assessed_at",
    "status",
    "reused_count",
    "collected_count",
    "packet",
    "full_explanation",
];
const FINDING: &[&str] = &[
    "schema",
    "finding_id",
    "task_digest",
    "check",
    "assessments",
    "packets",
    "explanations",
    "mcp",
    "collected_count",
    "reused_count",
];
impl McpGateway {
    pub fn new(
        server: ServerSpec,
        anchor: Value,
        mut options: GatewayOptions,
        on_call: Option<CallGate>,
    ) -> Result<Self, String> {
        server.validate()?;
        let task = validate_anchor(&anchor)?;
        let executable = options
            .host_paths
            .eal_mcp
            .canonicalize()
            .map_err(|_| "Native EAL MCP executable is unavailable")?;
        let directory = executable
            .parent()
            .ok_or("Native EAL MCP executable has no runtime directory")?;
        for (name, configured) in [
            ("eal-command-supervisor", &options.host_paths.supervisor),
            ("eal-method-worker", &options.host_paths.method_worker),
        ] {
            let sibling = directory
                .join(format!("{name}{}", std::env::consts::EXE_SUFFIX))
                .canonicalize()
                .map_err(|_| "Gateway companions must be installed beside eal-mcp")?;
            if configured
                .canonicalize()
                .map_err(|_| "Gateway companion is unavailable")?
                != sibling
            {
                return Err(
                    "Gateway declared companions differ from the eal-mcp runtime directory".into(),
                );
            }
        }
        options.host_paths.eal_mcp = executable;
        if options.operation_timeout.is_zero()
            || Instant::now()
                .checked_add(options.operation_timeout)
                .is_none()
        {
            return Err("MCP operation timeout must be positive and representable".into());
        }
        Ok(Self {
            server,
            anchor,
            task,
            options,
            on_call,
            calls: vec![],
        })
    }
    pub fn calls(&self) -> &[Value] {
        &self.calls
    }
    fn session(&self, exposure: &str) -> Result<PersistentMcp, String> {
        let settings = ServerSettings {
            workspace: self.server.workspace.clone(),
            registry: self.server.registry.clone(),
            database: self.server.database.clone(),
            methods: self
                .server
                .methods
                .clone()
                .unwrap_or_else(|| "builtin".into()),
            limits: self.server.limits.clone(),
            exposure: exposure.into(),
            known_entries: if exposure == "registered" {
                vec![self.task.entry_id.clone()]
            } else {
                vec![]
            },
            ..Default::default()
        };
        settings.validate()?;
        let target = host::stdio_target(
            &settings,
            &self.options.environment,
            self.options.host_paths.eal_mcp.clone(),
        );
        PersistentMcp::new(target, self.options.operation_timeout)
    }
    fn call(
        &mut self,
        session: &PersistentMcp,
        name: &str,
        arguments: Value,
        exposure: &str,
        deadline: Instant,
    ) -> Result<Value, String> {
        self.calls.push(
            json!({"name":name,"arguments":arguments,"exposure":exposure,"outcome":"not_called"}),
        );
        let index = self.calls.len() - 1;
        if let Some(gate) = &self.on_call {
            gate(name, &arguments, exposure)?;
        }
        check_deadline(deadline)?;
        self.calls[index]["outcome"] = json!("started");
        let result = session
            .call(name, &arguments, deadline)
            .map_err(|e| e.to_string())
            .and_then(|result| {
                if result.is_error.unwrap_or(false) {
                    return Err(format!(
                        "EAL MCP {name} did not return a successful structured result"
                    ));
                }
                result
                    .structured_content
                    .filter(Value::is_object)
                    .ok_or_else(|| {
                        format!("EAL MCP {name} did not return a successful structured result")
                    })
            });
        self.calls[index]["outcome"] = json!(if result.is_ok() {
            "success"
        } else {
            "interrupted_or_failed"
        });
        result
    }
    pub fn assess(&mut self, deadline: Instant) -> Result<Value, String> {
        check_deadline(deadline)?;
        let check = check_task_with_host(&self.server, &self.anchor, &self.options.host_paths)?;
        check_deadline(deadline)?;
        if check["ok"] != true {
            return make_finding(
                &self.anchor,
                &check,
                None,
                None,
                None,
                Some(&json!(self.calls)),
            );
        }
        let mut assessments = serde_json::Map::new();
        {
            let session = self.session("registered")?;
            let sources = self.call(&session, "eal_sources", json!({}), "registered", deadline)?;
            let entries = sources["sources"]
                .as_array()
                .ok_or("Actual MCP discovery differs from the pinned entry and selected claims")?;
            if entries.len() != 1
                || !entries[0].is_object()
                || entries[0]["entry_id"] != self.task.entry_id
                || entries[0]["source_digest"] != self.anchor["source_digest"]
            {
                return Err(
                    "Actual MCP discovery differs from the pinned entry and selected claims".into(),
                );
            }
            let claims = entries[0]["claims"]
                .as_array()
                .ok_or("Actual MCP discovery differs from the pinned entry and selected claims")?;
            let selected = claims
                .iter()
                .map(|v| {
                    v.as_str().ok_or(
                        "Actual MCP discovery differs from the pinned entry and selected claims",
                    )
                })
                .collect::<Result<BTreeSet<_>, _>>()?;
            if selected.len() != claims.len()
                || self
                    .task
                    .claims
                    .iter()
                    .any(|claim| !selected.contains(claim.as_str()))
            {
                return Err(
                    "Actual MCP discovery differs from the pinned entry and selected claims".into(),
                );
            }
            for claim in self.task.claims.clone() {
                check_deadline(deadline)?;
                let current =
                    check_task_with_host(&self.server, &self.anchor, &self.options.host_paths)?;
                if current["ok"] != true {
                    return Err("Task correspondence changed during registered assessment".into());
                }
                let row = self.call(
                    &session,
                    "eal_assess_known",
                    json!({"entry_id":self.task.entry_id,"claim":claim}),
                    "registered",
                    deadline,
                )?;
                assessments.insert(claim, row);
            }
            session.close();
        }
        let assessments = Value::Object(assessments);
        let (packets, explanations) = self.retrieve(&assessments, deadline)?;
        let final_check =
            check_task_with_host(&self.server, &self.anchor, &self.options.host_paths)?;
        check_deadline(deadline)?;
        if final_check["ok"] != true {
            return Err("Task correspondence changed during persisted assessment retrieval".into());
        }
        make_finding(
            &self.anchor,
            &check,
            Some(&assessments),
            Some(&packets),
            Some(&explanations),
            Some(&json!(self.calls)),
        )
    }
    fn retrieve(
        &mut self,
        assessments: &Value,
        deadline: Instant,
    ) -> Result<(Value, Value), String> {
        let rows = assessments
            .as_object()
            .ok_or("Finding assessments must be an object")?;
        let mut packets = serde_json::Map::new();
        let mut explanations = serde_json::Map::new();
        if rows.is_empty() {
            return Ok((Value::Object(packets), Value::Object(explanations)));
        }
        let session = self.session("operator")?;
        for (claim, row) in rows {
            let id = row["assessment_id"]
                .as_str()
                .ok_or("Registered assessment has no persisted identity")?;
            let arguments = json!({"assessment_id":id,"claim":claim});
            let packet = self.call(
                &session,
                "eal_packet",
                arguments.clone(),
                "operator",
                deadline,
            )?;
            let explanation =
                self.call(&session, "eal_explain", arguments, "operator", deadline)?;
            self.wrapper(claim, row, &packet, &explanation)?;
            packets.insert(claim.clone(), packet);
            explanations.insert(claim.clone(), explanation);
        }
        session.close();
        Ok((Value::Object(packets), Value::Object(explanations)))
    }
    fn wrapper(
        &self,
        claim: &str,
        row: &Value,
        packet: &Value,
        explanation: &Value,
    ) -> Result<(), String> {
        if !keys(row, WRAPPER) || row["schema"] != "EAL/registered-assessment/1" {
            return Err("Registered assessment wrapper differs from its closed contract".into());
        }
        if row["entry_id"] != self.task.entry_id
            || row["claim"] != claim
            || row["source_digest"] != self.anchor["source_digest"]
            || row["context_fingerprint"] != digest(&self.anchor["registered_context"])?
        {
            return Err("Registered assessment differs from the pinned task".into());
        }
        for name in ["collected_count", "reused_count"] {
            count(&row[name])
                .ok_or("Registered acquisition counts require nonnegative integers")?;
        }
        if !["supported", "contested", "unsupported", "out_of_scope"]
            .contains(&row["status"].as_str().unwrap_or(""))
        {
            return Err("Registered assessment contains an unknown EAL status".into());
        }
        let assessed = parse_time(
            row["assessed_at"]
                .as_str()
                .ok_or("Registered assessment has an invalid time")?,
        )?;
        if assessed
            < parse_time(
                self.anchor["pinned_at"]
                    .as_str()
                    .ok_or("Task has an invalid pin time")?,
            )?
        {
            return Err("Registered assessment predates this task pin".into());
        }
        if digest(&row["packet"])? != digest(packet)? {
            return Err(
                "Registered assessment packet differs from actual persisted MCP packet".into(),
            );
        }
        if row["full_explanation"] != packet["full_explanation"] {
            return Err(
                "Registered explanation reference differs from its persisted packet".into(),
            );
        }
        for (name, expected) in [
            ("assessment_id", &row["assessment_id"]),
            ("source_digest", &self.anchor["source_digest"]),
            ("context_fingerprint", &row["context_fingerprint"]),
            (
                "method_registry_fingerprint",
                &self.anchor["method_registry_fingerprint"],
            ),
        ] {
            if packet.get(name) != Some(expected) {
                return Err("Persisted packet identity differs from the task assessment".into());
            }
        }
        if packet["claims"][claim]["status"] != row["status"] {
            return Err("Persisted packet status differs from its registered assessment".into());
        }
        for (name, expected) in [
            ("assessment_id", row["assessment_id"].clone()),
            ("claim", json!(claim)),
            ("source_digest", self.anchor["source_digest"].clone()),
            (
                "method_registry_fingerprint",
                self.anchor["method_registry_fingerprint"].clone(),
            ),
            ("assessed_at", row["assessed_at"].clone()),
        ] {
            if explanation.get(name) != Some(&expected) {
                return Err(
                    "Persisted explanation identity differs from the task assessment".into(),
                );
            }
        }
        if explanation["result"]["status"] != row["status"] {
            return Err(
                "Persisted explanation status differs from its registered assessment".into(),
            );
        }
        Ok(())
    }
    pub fn lookup(&mut self, finding: &Value, deadline: Instant) -> Result<Value, String> {
        check_deadline(deadline)?;
        if !keys(finding, FINDING)
            || finding["schema"] != FINDING_SCHEMA
            || finding["task_digest"] != self.anchor["task_digest"]
        {
            return Err("Finding identity differs from its complete trusted content".into());
        }
        let mut content = finding.as_object().unwrap().clone();
        content.remove("finding_id");
        if finding["finding_id"] != format!("finding_{}", digest(&Value::Object(content))?) {
            return Err("Finding identity differs from its complete trusted content".into());
        }
        let current = check_task_with_host(&self.server, &self.anchor, &self.options.host_paths)?;
        if current != finding["check"] {
            return Err("Task correspondence differs from the issued finding".into());
        }
        let rows = finding["assessments"]
            .as_object()
            .ok_or("Finding assessments must be an object")?;
        let selected = self
            .task
            .claims
            .iter()
            .map(String::as_str)
            .collect::<BTreeSet<_>>();
        let names = rows.keys().map(String::as_str).collect::<BTreeSet<_>>();
        if !names.is_empty() && names != selected {
            return Err("Finding must contain either all selected assessments or none".into());
        }
        for key in ["packets", "explanations"] {
            if finding[key]
                .as_object()
                .is_none_or(|v| v.keys().map(String::as_str).collect::<BTreeSet<_>>() != names)
            {
                return Err("Finding assessment, packet and explanation selections differ".into());
            }
        }
        if current["ok"] != !rows.is_empty() {
            return Err("Finding assessments differ from its correspondence disposition".into());
        }
        for field in ["collected_count", "reused_count"] {
            let sum = rows
                .values()
                .try_fold(0u128, |sum, row| {
                    count(&row[field]).and_then(|value| sum.checked_add(value))
                })
                .ok_or("Finding acquisition counts require bounded nonnegative integers")?;
            if count(&finding[field]) != Some(sum) {
                return Err(
                    "Finding acquisition counts differ from its registered assessments".into(),
                );
            }
        }
        let (packets, explanations) = self.retrieve(&finding["assessments"], deadline)?;
        if digest(&packets)? != digest(&finding["packets"])?
            || digest(&explanations)? != digest(&finding["explanations"])?
        {
            return Err("Finding differs from actual persisted MCP packets or explanations".into());
        }
        if check_task_with_host(&self.server, &self.anchor, &self.options.host_paths)? != current {
            return Err("Task correspondence changed during lookup".into());
        }
        check_deadline(deadline)?;
        Ok(finding.clone())
    }
}
fn keys(value: &Value, expected: &[&str]) -> bool {
    value
        .as_object()
        .is_some_and(|v| v.len() == expected.len() && expected.iter().all(|k| v.contains_key(*k)))
}
fn digest(value: &Value) -> Result<String, String> {
    canonical::canonical_digest(
        value,
        &ExecutionLimits {
            json_depth: 128,
            record_bytes: 64 * 1024 * 1024,
            ..Default::default()
        },
    )
}
fn count(value: &Value) -> Option<u128> {
    let raw = value.as_number()?.to_string();
    if raw.contains(['.', 'e', 'E']) {
        return None;
    }
    raw.parse().ok()
}
fn check_deadline(deadline: Instant) -> Result<(), String> {
    if Instant::now() >= deadline {
        Err("MCP gateway exceeded its total deadline; reconcile before retrying".into())
    } else {
        Ok(())
    }
}
