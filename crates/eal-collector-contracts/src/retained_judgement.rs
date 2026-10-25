//! Explicit reviewer records: correspondence is checkable; their truth is not certified.
use crate::capability::SubjectScope;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const RETAINED_JUDGEMENT_SCHEMA: &str = "eal.retained-judgement/1";
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum JudgementVerdict {
    Supported,
    Rejected,
    Inconclusive,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RetainedJudgement {
    pub schema: String,
    pub criterion: String,
    pub scope: SubjectScope,
    pub reviewer_identity: String,
    pub reviewed_at: String,
    pub verdict: JudgementVerdict,
    pub reasons: Vec<String>,
    #[serde(deserialize_with = "crate::deserialize_value_exact")]
    #[schemars(with = "Value")]
    pub provenance: Value,
}
impl RetainedJudgement {
    pub fn validate(&self) -> Result<(), String> {
        self.scope.validate()?;
        if self.schema != RETAINED_JUDGEMENT_SCHEMA
            || !text(&self.criterion, 8192)
            || !text(&self.reviewer_identity, 1024)
            || self.reasons.is_empty()
            || self.reasons.len() > 128
            || self.reasons.iter().any(|reason| !text(reason, 8192))
            || !self.provenance.as_object().is_some_and(|fields| {
                !fields.is_empty()
                    && fields.len() <= 128
                    && fields.iter().all(|(key, value)| {
                        text(key, 1024)
                            && !value.is_null()
                            && !value.as_str().is_some_and(|text| text.trim().is_empty())
                    })
            })
        {
            return Err("retained judgement requires its versioned criterion/scope, reviewer identity, reasons and provenance".into());
        }
        chrono::DateTime::parse_from_rfc3339(&self.reviewed_at)
            .map_err(|_| "retained judgement reviewed_at requires RFC3339")?;
        crate::capability::canonical_identity(&self.provenance)?;
        Ok(())
    }
}
fn text(value: &str, maximum: usize) -> bool {
    !value.trim().is_empty() && value.len() <= maximum && !value.contains('\0')
}
pub fn retained_judgement_schema() -> Value {
    serde_json::to_value(schemars::schema_for!(RetainedJudgement))
        .expect("retained judgement schema serializes")
}
