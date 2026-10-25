//! Consume retained reviewer records without converting them into independent proof.
use crate::{
    execution::{SourceRecord, StepResult},
    plan::RetainedJudgement,
};
use eal_collector_contracts::capability::SubjectScope;
use serde_json::Value;
use std::collections::BTreeMap;

pub(crate) fn correspondence(
    criterion: &str,
    scope: &SubjectScope,
    values: &[Value],
    steps: &[&StepResult],
    sources: &BTreeMap<String, SourceRecord>,
) -> Result<RetainedJudgement, String> {
    let [selected] = values else {
        return Err(
            "scoped judgement requires one explicitly selected retained reviewer record".into(),
        );
    };
    let record: RetainedJudgement = serde_json::from_value(selected.clone())
        .map_err(|_| "retained judgement record has invalid fields")?;
    record.validate()?;
    if record.criterion != criterion || record.scope != *scope {
        return Err(
            "retained judgement criterion or subject scope differs from the requirement".into(),
        );
    }
    let reviewed = chrono::DateTime::parse_from_rfc3339(&record.reviewed_at)
        .map_err(|_| "retained judgement time is invalid")?;
    let source_ids = steps
        .iter()
        .flat_map(|step| step.observation_ids.iter())
        .collect::<std::collections::BTreeSet<_>>();
    if source_ids.is_empty() || source_ids.iter().any(|id| !sources.contains_key(*id)) {
        return Err("retained judgement has no complete qualified observation lineage".into());
    }
    let mut matched = false;
    let mut mismatch = None;
    for id in source_ids {
        let source = &sources[id];
        source.source.validate()?;
        if contains(&source.value, selected) {
            let declares_judgement = |field: &str| {
                source
                    .provenance
                    .get(field)
                    .and_then(Value::as_array)
                    .is_some_and(|values| {
                        values
                            .iter()
                            .any(|value| value.as_str() == Some("judgement"))
                    })
            };
            if !declares_judgement("qualified_support_types")
                || !declares_judgement("qualified_logical_strengths")
            {
                mismatch = Some(
                    "retained judgement source is not qualified to supply reviewer judgements"
                        .into(),
                );
                continue;
            }

            let observed = chrono::DateTime::parse_from_rfc3339(&source.observed_at)
                .map_err(|_| "retained judgement source time is invalid")?;
            if observed != reviewed {
                mismatch=Some("retained judgement reviewed_at differs from its original source observation time".to_owned());
                continue;
            }
            if scope
                .selectors
                .iter()
                .any(|(key, value)| source.source.scope.get(key) != Some(value))
            {
                mismatch=Some("retained judgement source identity differs from the expected subject selectors".to_owned());
                continue;
            }
            if source
                .provenance
                .as_object()
                .is_none_or(|fields| fields.is_empty())
            {
                mismatch = Some(
                    "retained judgement qualified source provenance is unavailable".to_owned(),
                );
                continue;
            }
            matched = true;
        }
    }
    if !matched {
        return Err(mismatch.unwrap_or_else(|| {
            "retained judgement is absent from its qualified source records".into()
        }));
    }
    Ok(record)
}
fn contains(root: &Value, selected: &Value) -> bool {
    let mut pending = vec![root];
    while let Some(value) = pending.pop() {
        if value == selected {
            return true;
        }
        match value {
            Value::Array(values) => pending.extend(values),
            Value::Object(fields) => pending.extend(fields.values()),
            _ => {}
        }
    }
    false
}
