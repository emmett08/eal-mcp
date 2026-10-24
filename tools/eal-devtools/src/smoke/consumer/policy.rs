//! Validate complete scenario coverage and the current downstream import policy.
use super::input::{array, load, required_string};
use super::{MAX_ARGUMENTS, MAX_BYTES, MAX_DEFEATS, Result};
use serde_json::Value;
use std::{collections::BTreeSet, path::Path};
pub(super) fn check_schema_and_policy(
    root: &Path,
    views: &[Value],
    explicit: &[Value],
    compiled: &[Value],
) -> Result<()> {
    let expected = explicit
        .iter()
        .map(|c| required_string(c, "name").map(|s| format!("explicit-{s}")))
        .chain(
            compiled
                .iter()
                .map(|c| required_string(c, "name").map(|s| format!("compiled-{s}"))),
        )
        .collect::<Result<BTreeSet<_>>>()?;
    let names = views
        .iter()
        .map(|c| required_string(c, "name").map(str::to_owned))
        .collect::<Result<Vec<_>>>()?;
    if names.len() != expected.len() || names.iter().cloned().collect::<BTreeSet<_>>() != expected {
        return Err("Views must cover exactly the authored native scenarios".into());
    }
    let schema = load(&root.join("contracts/schemas/aspic-view.schema.json"))?;
    let validator = jsonschema::options()
        .with_draft(jsonschema::Draft::Draft202012)
        .build(&schema)
        .map_err(|e| e.to_string())?;
    for case in views {
        let name = required_string(case, "name")?;
        let view = case.get("view").ok_or("Missing view")?;
        validator
            .validate(view)
            .map_err(|e| format!("{name}: {e}"))?;
        if serde_json::to_vec(view).map_err(|e| e.to_string())?.len() > MAX_BYTES {
            return Err(format!("{name}: view exceeds consumer byte policy"));
        }
        for (field, limit_field, policy) in [
            ("arguments", "formal_arguments", MAX_ARGUMENTS),
            ("defeats", "formal_defeats", MAX_DEFEATS),
        ] {
            let bound = view["execution_limits"][limit_field]
                .as_u64()
                .ok_or("Missing execution limit")?
                .min(policy);
            if array(view, field)?.len() as u64 > bound {
                return Err(format!("{name}: graph exceeds consumer {field} policy"));
            }
        }
    }
    Ok(())
}
