//! Native method wrapper for the shared pure verification API.
use super::validation::result;
use crate::ExecutionLimits;
use serde_json::{Value, json};

pub(super) fn assess(payload: &Value, limits: &ExecutionLimits) -> Result<Value, String> {
    let report = match crate::verification::evaluate_query(payload, limits) {
        Ok(report) => report,
        Err(error) if error.contains("budget") || error.contains("resource limit") => {
            return Ok(json!({"status":"incomplete","reasons":[error],"details":{}}));
        }
        Err(error) => return Err(error),
    };
    let details =
        json!({"verified":report.status==crate::verification::Status::Pass,"verification":report});
    // Unknown source meaning is a completed verification finding, not an
    // exhausted calculation. Consumers must inspect the report status. This
    // preserves independent graph routes and keeps unresolved reports visible.
    Ok(result(
        true,
        "Evaluated the declared source grounding and applicability obligations",
        details,
    ))
}
