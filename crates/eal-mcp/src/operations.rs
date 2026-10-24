//! Immutable operation catalogue shared by SDK dispatch and the text host.
use eal_runtime::{
    catalogue::{FindOptions, WorkspaceKnowledgeCatalogue},
    registered_assessment::{RegisteredAssessmentHost, RegisteredAssessmentOptions},
    service::{FormalOptions, ReasoningService},
};
use rmcp::model::Tool;
use serde_json::{Value, json};
use std::sync::Arc;

mod requests;
pub use requests::*;

pub fn operation_fields(name: &str) -> Option<OperationFields> {
    Operation::parse(name).map(Operation::fields)
}
pub fn is_acquisition_operation(name: &str) -> bool {
    Operation::parse(name).is_some_and(Operation::acquires)
}

pub(crate) struct Operations {
    service: Arc<ReasoningService>,
    selected: Vec<String>,
    catalogue: Option<Arc<WorkspaceKnowledgeCatalogue>>,
    host: Option<RegisteredAssessmentHost>,
    pub tools: Vec<Tool>,
}
impl Operations {
    pub fn new(
        service: Arc<ReasoningService>,
        selected: Vec<String>,
        exposure: Option<&str>,
    ) -> Result<Self, String> {
        let exposure = Exposure::resolve(exposure.filter(|name| !name.is_empty()), &selected)?;
        let catalogue = if selected.is_empty() {
            None
        } else {
            Some(Arc::new(WorkspaceKnowledgeCatalogue::new(Arc::clone(
                &service,
            ))?))
        };
        if let Some(catalogue) = &catalogue {
            for identifier in &selected {
                catalogue.get(identifier, false)?;
            }
        }
        let host = catalogue
            .as_ref()
            .map(|catalogue| {
                RegisteredAssessmentHost::new(Arc::clone(&service), Arc::clone(catalogue))
            })
            .transpose()?;
        let tools = OPERATIONS
            .iter()
            .copied()
            .filter(|operation| operation.exposure() == exposure)
            .map(Operation::tool)
            .collect();
        Ok(Self {
            service,
            selected,
            catalogue,
            host,
            tools,
        })
    }

    /// Decode before permits, leases, catalogue lookup or acquisition. The typed
    /// request cannot change between admission and its exhaustive handler match.
    pub fn decode(&self, name: &str, arguments: &Value) -> Result<OperationRequest, String> {
        if !self.tools.iter().any(|tool| tool.name == name) {
            return Err(
                "Unknown tool: operation is not exposed by this server [EAL_OPERATION_EXPOSURE]"
                    .into(),
            );
        }
        let request = OperationRequest::decode(name, arguments)?;
        match &request {
            OperationRequest::AssessKnown(request) => self.require_selected(&request.entry_id)?,
            OperationRequest::GetKnownPacket(request) => {
                self.require_selected(&request.entry_id)?
            }
            _ => (),
        }
        Ok(request)
    }

    pub fn dispatch(&self, request: OperationRequest) -> Result<Value, String> {
        let lease = request
            .operation()
            .acquires()
            .then(|| self.service.acquisition_lease())
            .transpose()?;
        match request {
            OperationRequest::Describe(_) => self.service.describe(),
            OperationRequest::Format(request) => self.service.format(&request.source),
            OperationRequest::Validate(request) => Ok(self.service.validate(&request.source)),
            OperationRequest::Plan(request) => self.service.plan(&request.source, &request.claim),
            OperationRequest::Collect(request) => self.service.collect_with_lease(
                &request.source,
                &Value::Object(request.context),
                request.evidence_ids.as_deref(),
                lease.ok_or("Missing acquisition lease")?,
            ),
            OperationRequest::CollectClaim(request) => self.service.collect_claim_with_lease(
                &request.source,
                &Value::Object(request.context),
                &request.claim,
                lease.ok_or("Missing acquisition lease")?,
            ),
            OperationRequest::Reason(request) => self.service.reason(
                &request.source,
                &Value::Object(request.context),
                request.collection_id.as_deref(),
                request.now.as_deref(),
            ),
            OperationRequest::CompileAspic(request) => self.service.compile_aspic(
                &request.source,
                &Value::Object(request.context),
                &request.collection_id,
                FormalOptions {
                    goal: &request.goal,
                    now: request.now.as_deref(),
                    semantics: request.semantics.as_deref().unwrap_or(DEFAULT_SEMANTICS),
                    query_mode: request.query_mode.as_deref().unwrap_or(DEFAULT_QUERY_MODE),
                    preference: request.preference.as_deref(),
                },
            ),
            OperationRequest::Explain(request) => self
                .service
                .explain(&request.assessment_id, request.claim.as_deref()),
            OperationRequest::Packet(request) => self.service.packet_with_limits(
                &request.assessment_id,
                request.claim.as_deref(),
                request.packet_limits.unwrap_or_default().resolve()?,
            ),
            OperationRequest::Grounded(request) => eal_core::dialectic::solve_grounded(
                &json!(request.arguments),
                &json!(request.attacks),
                self.service.limits(),
            ),
            OperationRequest::Sources(request) => {
                let limit = request.limit.unwrap_or(DEFAULT_LIMIT);
                let offset = request.offset.unwrap_or(DEFAULT_OFFSET);
                let catalogue = self
                    .catalogue
                    .as_ref()
                    .ok_or("Missing registered catalogue")?;
                let mut entries = self
                    .selected
                    .iter()
                    .map(|identifier| catalogue.get(identifier, false))
                    .collect::<Result<Vec<_>, _>>()?;
                entries.sort_by(|left, right| left["path"].as_str().cmp(&right["path"].as_str()));
                Ok(
                    json!({"sources": entries.iter().skip(offset).take(limit).map(source_summary).collect::<Vec<_>>(), "limit": limit, "offset": offset}),
                )
            }
            OperationRequest::FindClaims(request) => {
                let limit = request.limit.unwrap_or(DEFAULT_LIMIT);
                let catalogue = self
                    .catalogue
                    .as_ref()
                    .ok_or("Missing registered catalogue")?;
                let mut matches = Vec::new();
                for identifier in &self.selected {
                    let entry = catalogue.get(identifier, false)?;
                    matches.extend(catalogue.find(FindOptions {
                        query: request.query.as_deref(),
                        claim: request.claim.as_deref(),
                        path: entry["path"].as_str(),
                        limit: 1,
                        ..FindOptions::default()
                    })?);
                }
                matches.sort_by(|left, right| left["path"].as_str().cmp(&right["path"].as_str()));
                Ok(
                    json!({"matches": matches.iter().take(limit).map(source_summary).collect::<Vec<_>>()}),
                )
            }
            OperationRequest::AssessKnown(request) => self
                .host
                .as_ref()
                .ok_or("Missing registered assessment host")?
                .assess_with_lease(
                    &request.entry_id,
                    &request.claim,
                    RegisteredAssessmentOptions {
                        packet_limits: Some(&request.packet_limits.unwrap_or_default().resolve()?),
                        ..Default::default()
                    },
                    lease.ok_or("Missing acquisition lease")?,
                ),
            OperationRequest::GetKnownPacket(request) => self
                .host
                .as_ref()
                .ok_or("Missing registered assessment host")?
                .retrieve_packet(
                    &request.entry_id,
                    &request.assessment_id,
                    &request.claim,
                    request.packet_limits.unwrap_or_default().resolve()?,
                ),
        }
    }
    fn require_selected(&self, identifier: &str) -> Result<(), String> {
        self.selected
            .iter()
            .any(|selected| selected == identifier)
            .then_some(())
            .ok_or_else(|| "Registered source is not exposed by this server".into())
    }
}
fn source_summary(entry: &Value) -> Value {
    let mut result = serde_json::Map::new();
    for key in [
        "entry_id",
        "path",
        "label",
        "source_digest",
        "claims",
        "claim_metadata",
    ] {
        result.insert(key.into(), entry[key].clone());
    }
    Value::Object(result)
}
