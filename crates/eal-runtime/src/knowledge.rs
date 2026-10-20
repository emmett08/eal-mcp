//! Application façade and checked context for a text model without tool use.
use crate::{
    catalogue::{FindOptions, RegisterOptions, WorkspaceKnowledgeCatalogue},
    model_context::{CONTEXT_INSTRUCTION, ModelContextBuilder},
    registered_assessment::{RegisteredAssessmentHost, RegisteredAssessmentOptions},
    service::{ReasoningService, ServiceConfig},
};
use eal_core::canonical;
use serde_json::{Value, json};
use std::sync::Arc;

#[derive(Clone)]
pub struct EALKnowledgeBase {
    pub service: Arc<ReasoningService>,
    pub catalogue: Arc<WorkspaceKnowledgeCatalogue>,
    pub host: RegisteredAssessmentHost,
}
impl EALKnowledgeBase {
    pub fn new(service: Arc<ReasoningService>) -> Result<Self, String> {
        let catalogue = Arc::new(WorkspaceKnowledgeCatalogue::new(Arc::clone(&service))?);
        let host = RegisteredAssessmentHost::new(Arc::clone(&service), Arc::clone(&catalogue))?;
        Ok(Self {
            service,
            catalogue,
            host,
        })
    }
    pub fn from_config(config: ServiceConfig) -> Result<Self, String> {
        Self::new(Arc::new(ReasoningService::new(config)?))
    }
    pub fn register(&self, path: &str, options: RegisterOptions<'_>) -> Result<Value, String> {
        self.catalogue.register(path, options)
    }
    pub fn register_tree(
        &self,
        directory: &str,
        context: Option<&Value>,
        limit: usize,
    ) -> Result<Value, String> {
        self.catalogue.register_tree(directory, context, limit)
    }
    pub fn sources(&self, limit: usize, offset: usize) -> Result<Vec<Value>, String> {
        self.catalogue.list(limit, offset)
    }
    pub fn find(&self, options: FindOptions<'_>) -> Result<Vec<Value>, String> {
        self.catalogue.find(options)
    }
    pub fn assess(
        &self,
        entry_id: &str,
        claim: &str,
        options: RegisteredAssessmentOptions<'_>,
    ) -> Result<Value, String> {
        self.host.assess(entry_id, claim, options)
    }
    pub fn history(
        &self,
        entry_id: &str,
        source_digest: Option<&str>,
        limit: usize,
    ) -> Result<Value, String> {
        self.host.history(entry_id, source_digest, limit)
    }
    pub fn explain(&self, assessment_id: &str, claim: Option<&str>) -> Result<Value, String> {
        self.service.explain(assessment_id, claim)
    }
}

#[derive(Clone)]
pub struct ModelContextAdapter {
    pub knowledge: Arc<EALKnowledgeBase>,
}
impl ModelContextAdapter {
    pub fn new(knowledge: Arc<EALKnowledgeBase>) -> Self {
        Self { knowledge }
    }
    pub fn prepare(
        &self,
        question: &str,
        entry_id: &str,
        claim: &str,
        options: RegisteredAssessmentOptions<'_>,
    ) -> Result<Value, String> {
        // UTF-8 byte admission precedes every catalogue lookup and acquisition.
        if question
            .trim_matches(|c: char| c.is_whitespace() || matches!(c, '\u{1c}'..='\u{1f}'))
            .is_empty()
            || question.len() > 8192
        {
            return Err("question must contain one to 8192 UTF-8 bytes".into());
        }
        let assessment = self.knowledge.assess(entry_id, claim, options)?;
        let context = ModelContextBuilder::new().build(&assessment)?;
        let compact = String::from_utf8(canonical::canonical_json_bytes(
            &context,
            self.knowledge.service.limits(),
        )?)
        .map_err(|e| e.to_string())?;
        Ok(
            json!({"schema":"EAL/model-context/2","assessment":assessment,"context":context,
            "messages":[{"role":"system","content":format!("{CONTEXT_INSTRUCTION}{compact}")},{"role":"user","content":question}]}),
        )
    }
}
