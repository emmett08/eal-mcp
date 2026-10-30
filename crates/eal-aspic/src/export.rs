//! Recompute formal semantics before exporting supplied, unauthenticated provenance.
use eal_core::{ExecutionLimits, canonical, methods};
use serde_json::{Value, json};

mod provenance;
mod schema;
mod verification;

pub const VIEW_VERSION: &str = "aspic-view/3";
pub const MAX_INPUT_BYTES: usize = 4 * 1024 * 1024;

/// Export only a complete result exactly reproduced under its captured budgets.
///
/// Captured budgets may tighten, but cannot raise, the exporter's host policy.
/// Source-map assertions are checked for shape and correspondence, not authenticity.
pub fn export_aspic_view(result: &Value, limits: &ExecutionLimits) -> Result<Value, String> {
    limits.validate()?;
    input_bound(result, limits)?;
    let object = result
        .as_object()
        .ok_or("Export requires a theory and its formal result")?;
    let theory = object
        .get("theory")
        .filter(|v| v.is_object())
        .ok_or("Export requires a theory and its formal result")?;
    let formal = object.get("formal").unwrap_or(&Value::Null);
    let source = match object.get("source_map") {
        Some(source) if source.is_object() => Some(source),
        Some(_) => return Err("Invalid source map structure: Source map must be an object".into()),
        None => None,
    };
    let selected: ExecutionLimits = match source.and_then(|s| s.get("execution_limits")) {
        Some(value) => serde_json::from_value(value.clone())
            .map_err(|_| "Invalid snapshot execution budgets")?,
        None => limits.clone(),
    };
    let policy = limits.describe();
    let snapshot = selected.describe();
    if snapshot
        .as_object()
        .expect("limits object")
        .iter()
        .any(|(key, value)| value.as_u64() > policy[key].as_u64())
    {
        return Err(
            "Snapshot budgets exceed the exporter host policy; configure explicit host limits"
                .into(),
        );
    }
    let errors =
        methods::schema_errors_with_limits(formal, crate::registry::output_schema(), "$", limits);
    if !errors.is_empty() {
        return Err(format!(
            "Invalid ASPIC+ result: {}",
            errors.into_iter().take(4).collect::<Vec<_>>().join("; ")
        ));
    }
    if canonical::canonical_digest(theory, limits)?
        != formal["theory_sha256"].as_str().unwrap_or("")
    {
        return Err("Formal result does not match the supplied theory digest".into());
    }
    let payload = json!({"theory":theory});
    let contract = crate::registry::aspic_contract();
    if canonical::canonical_json_bytes(&payload, &selected)?.len() > contract.max_input_bytes {
        return Err("Bounded ASPIC recomputation failed: Method input exceeds byte limit".into());
    }
    let recomputed = crate::solve_aspic(&payload, &selected)
        .map_err(|error| format!("Bounded ASPIC recomputation failed: {error}"))?;
    crate::registry::check_output_bytes(&recomputed, &selected)
        .map_err(|error| format!("Bounded ASPIC recomputation failed: {error}"))?;
    if canonical::canonical_digest(formal, limits)?
        != canonical::canonical_digest(&recomputed, limits)?
    {
        return Err("Supplied formal result differs from bounded ASPIC recomputation".into());
    }
    if let Some(source) = source {
        schema::validate(source, schema::source_schema(), limits)
            .map_err(|error| format!("Invalid source map structure: {}", error.message))?;
    }
    let provenance = provenance::Origins::check(result, theory, formal, source)?;
    let nodes = formal["arguments"]
        .as_array()
        .expect("validated formal arguments")
        .iter()
        .map(|arg| {
            let mut node = arg.clone();
            let origin = arg.get("rule_id").and_then(Value::as_str)
            .and_then(|id| provenance.rules.get(id))
            .or_else(|| if arg.get("rule_id").is_none() {
                provenance.atoms.get(arg["conclusion"].as_str().expect("validated conclusion"))
            } else { None })
            .cloned().unwrap_or_else(|| json!({
                "kind":if arg.get("rule_id").is_some() {"formal rule"} else {"formal premise"},
                "name":arg.get("rule_id").unwrap_or(&arg["conclusion"])
            }));
            let claim = provenance
                .claims
                .get(arg["conclusion"].as_str().expect("validated conclusion"));
            node["display_conclusion"] = if let Some(claim) = claim {
                claim["name"].clone()
            } else if matches!(
                origin["kind"].as_str(),
                Some("evidence" | "assumption" | "objection")
            ) {
                origin["name"].clone()
            } else {
                arg["conclusion"].clone()
            };
            node["origin"] = origin;
            if let Some(claim) = claim {
                node["authored_statement"] = claim["statement"].clone();
            }
            node
        })
        .collect::<Vec<_>>();
    let by_id = formal["arguments"]
        .as_array()
        .expect("validated arguments")
        .iter()
        .map(|arg| (arg["id"].as_str().expect("validated id"), arg))
        .collect::<std::collections::BTreeMap<_, _>>();
    let defeats = formal["defeats"]
        .as_array()
        .expect("validated defeats")
        .iter()
        .map(|event| {
            let mut event = event.clone();
            let target = by_id[event["subargument"]
                .as_str()
                .expect("validated subargument")];
            let attacked = if event["kind"] == "undercut" {
                &target["rule_name"]
            } else {
                &target["conclusion"]
            };
            let attacker =
                &by_id[event["attacker"].as_str().expect("validated attacker")]["conclusion"];
            if let Some(origin) = provenance.contraries.get(&(
                attacker.as_str().unwrap_or("").to_owned(),
                attacked.as_str().unwrap_or("").to_owned(),
            )) {
                event["relation_origin"] = origin.clone();
            }
            event
        })
        .collect::<Vec<_>>();
    let mut view = json!({
        "schema":VIEW_VERSION,"theory_digest":formal["theory_sha256"],
        "evaluated_at":source.and_then(|s|s.get("assessed_at")),
        "validation":{"formal_result":"recomputed","provenance":"supplied"},
        "profile":result.get("profile").cloned().unwrap_or_else(||json!("supplied-formal-result")),
        "goal":theory["goal"],
        "goal_claim":source.map(|s|&s["goal"]["claim"]).or_else(||result.get("claim")),
        "formal_status":formal["grounded_status"],
        "execution_limits":snapshot,
        "formal_directives":source.and_then(|s|s.get("formal_directives")).cloned().unwrap_or_else(||json!([])),
        "claim_status":result.get("claim_status"),"authored_claim_status":result.get("authored_claim_status"),
        "source_digest":result.get("source_digest"),
        "snapshot_digest":source.and_then(|s|s.get("snapshot_digest")),
        "arguments":nodes,"defeats":defeats,"unavailable_evidence":provenance.missing
    });
    let verification_reports = verification::reports(source);
    if !verification_reports.is_empty() {
        view["verification_reports"] = json!(verification_reports);
    }
    for key in [
        "semantics",
        "query_mode",
        "query_status",
        "preference",
        "extensions",
    ] {
        view[key] = formal[key].clone();
    }
    schema::validate(&view, schema::view_schema(), limits).map_err(|error| {
        format!(
            "Invalid ASPIC+ view metadata at {}: {}",
            error.path, error.message
        )
    })?;
    Ok(view)
}

/// Bound before cloning, serialising or recursively walking caller-supplied data.
/// Size uses Python json.dumps' UTF-8/default-separator representation.
fn input_bound(value: &Value, limits: &ExecutionLimits) -> Result<(), String> {
    let mut pending = vec![(value, 0usize)];
    let mut nodes = 0usize;
    let mut bytes = 0usize;
    while let Some((value, depth)) = pending.pop() {
        nodes += 1;
        if depth > limits.json_depth.min(128) || nodes > limits.json_nodes {
            return Err("ASPIC+ export requires bounded JSON data".into());
        }
        let size = match value {
            Value::Null => 4,
            Value::Bool(true) => 4,
            Value::Bool(false) => 5,
            Value::String(value) => string_size(value),
            Value::Number(value) => {
                if value.to_string().len() > MAX_INPUT_BYTES {
                    return Err("ASPIC+ result exceeds the 4 MiB export input limit".into());
                }
                canonical::canonical_json_bytes(&Value::Number(value.clone()), limits)?.len()
            }
            Value::Array(values) => {
                if nodes
                    .saturating_add(pending.len())
                    .saturating_add(values.len())
                    > limits.json_nodes
                {
                    return Err("ASPIC+ export requires bounded JSON data".into());
                }
                pending.extend(values.iter().map(|v| (v, depth + 1)));
                2 + values.len().saturating_sub(1) * 2
            }
            Value::Object(values) => {
                if nodes
                    .saturating_add(pending.len())
                    .saturating_add(values.len())
                    > limits.json_nodes
                {
                    return Err("ASPIC+ export requires bounded JSON data".into());
                }
                pending.extend(values.values().map(|v| (v, depth + 1)));
                let mut size = 2 + values.len().saturating_sub(1) * 2;
                for key in values.keys() {
                    size = size.saturating_add(string_size(key) + 2);
                    if size > MAX_INPUT_BYTES {
                        return Err("ASPIC+ result exceeds the 4 MiB export input limit".into());
                    }
                }
                size
            }
        };
        bytes = bytes.saturating_add(size);
        if bytes > MAX_INPUT_BYTES {
            return Err("ASPIC+ result exceeds the 4 MiB export input limit".into());
        }
    }
    Ok(())
}

fn string_size(text: &str) -> usize {
    if text.len() > MAX_INPUT_BYTES {
        return MAX_INPUT_BYTES + 1;
    }
    2 + text
        .chars()
        .map(|c| match c {
            '"' | '\\' | '\u{08}' | '\u{0c}' | '\n' | '\r' | '\t' => 2,
            c if (c as u32) < 32 => 6,
            c => c.len_utf8(),
        })
        .sum::<usize>()
}
