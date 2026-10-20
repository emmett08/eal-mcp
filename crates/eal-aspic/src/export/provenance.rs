//! Supplied source-map correspondence. These checks do not authenticate authorship.
use eal_core::expressions::numbers;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Default)]
pub struct Origins {
    pub rules: BTreeMap<String, Value>,
    pub atoms: BTreeMap<String, Value>,
    pub claims: BTreeMap<String, Value>,
    pub contraries: BTreeMap<(String, String), Value>,
    pub missing: Vec<Value>,
}

impl Origins {
    pub fn check(
        result: &Value,
        theory: &Value,
        formal: &Value,
        source: Option<&Value>,
    ) -> Result<Self, String> {
        let mut out = Self::default();
        let Some(source) = source else {
            return Ok(out);
        };
        let goal = source
            .get("goal")
            .filter(|v| v.is_object())
            .ok_or("Source map does not match the supplied theory and goal")?;
        if source.get("theory_digest") != formal.get("theory_sha256")
            || goal.get("atom") != theory.get("goal")
            || result.get("claim").unwrap_or(&goal["claim"]) != &goal["claim"]
            || result
                .get("source_digest")
                .unwrap_or(&source["source_digest"])
                != &source["source_digest"]
        {
            return Err("Source map does not match the supplied theory and goal".into());
        }
        if result
            .get("snapshot_digest")
            .unwrap_or(&source["snapshot_digest"])
            != &source["snapshot_digest"]
        {
            return Err("Snapshot digest disagrees with the source map".into());
        }
        let premises = theory["premises"]
            .as_array()
            .expect("recomputed theory premises")
            .iter()
            .map(|p| (p["atom"].as_str().expect("validated premise atom"), p))
            .collect::<BTreeMap<_, _>>();
        let rules = theory["rules"]
            .as_array()
            .expect("recomputed theory rules")
            .iter()
            .map(|r| (r["id"].as_str().expect("validated rule id"), r))
            .collect::<BTreeMap<_, _>>();
        for (name, item) in entries(source, "claims")? {
            let atom = item["atom"]
                .as_str()
                .filter(|atom| !out.claims.contains_key(*atom))
                .ok_or("Source map claim identity is inconsistent")?;
            if !item["statement"].is_string() {
                return Err("Source map claim identity is inconsistent".into());
            }
            out.claims.insert(
                atom.into(),
                json!({"name":name,"statement":item["statement"]}),
            );
        }
        if !goal["claim"].is_string()
            || out
                .claims
                .get(theory["goal"].as_str().expect("validated goal"))
                .map(|v| &v["name"])
                != Some(&goal["claim"])
        {
            return Err("Source map goal claim does not name the formal goal atom".into());
        }
        for kind in ["arguments", "assumptions", "objections"] {
            for (name, item) in entries(source, kind)? {
                if item["emitted"] != true {
                    continue;
                }
                let id = item["rule_id"]
                    .as_str()
                    .ok_or("Source map rule identity is inconsistent")?;
                let rule = rules
                    .get(id)
                    .filter(|_| !out.rules.contains_key(id))
                    .ok_or("Source map rule identity is inconsistent")?;
                if kind == "arguments"
                    && (!item["reasoning"].is_string() || !item["rationale"].is_string())
                {
                    return Err("Source map rule identity is inconsistent".into());
                }
                if item.get("conclusion_atom").unwrap_or(&item["atom"]) != &rule["consequent"]
                    || item.get("rule_kind").unwrap_or(&rule["kind"]) != &rule["kind"]
                    || item["applicability_atom"] != rule["name"]
                    || !python_equal(&item["rank"], &rule["rank"])
                {
                    return Err("Source map rule fields disagree with the formal theory".into());
                }
                let review = item
                    .get("strict_annotation")
                    .filter(|v| !v.is_null())
                    .unwrap_or(&item["rank_annotation"]);
                let mut origin = json!({
                    "kind":&kind[..kind.len()-1],"name":name,"span":item["span"],"formal_review":review
                });
                if kind == "arguments" {
                    origin["rationale"] = item["rationale"].clone();
                    origin["reasoning"] = item["reasoning"].clone();
                }
                for key in ["source_file", "scope_transfer"] {
                    if let Some(value) = item.get(key) {
                        origin[key] = value.clone();
                    }
                }
                out.rules.insert(id.into(), origin);
            }
        }
        for (name, item) in entries(source, "evidence")? {
            if item["available"] == true {
                let atom = item["atom"]
                    .as_str()
                    .ok_or("Source map evidence identity is inconsistent")?;
                let premise = premises
                    .get(atom)
                    .filter(|p| p["kind"] == "ordinary")
                    .filter(|_| !out.atoms.contains_key(atom))
                    .ok_or("Source map evidence identity is inconsistent")?;
                if !python_equal(&item["rank"], &premise["rank"]) {
                    return Err("Source map evidence identity is inconsistent".into());
                }
                let mut origin = json!({"kind":"evidence","name":name,"span":item["span"],
                    "formal_review":item["rank_annotation"],"observation":item["identity"]});
                if let Some(value) = item.get("source_file") {
                    origin["source_file"] = value.clone();
                }
                out.atoms.insert(atom.into(), origin);
            } else {
                let reasons = item.get("reasons").cloned().unwrap_or_else(|| json!([]));
                if !reasons
                    .as_array()
                    .is_some_and(|v| v.iter().all(Value::is_string))
                {
                    return Err("Unavailable evidence reasons must be text".into());
                }
                let issues = item
                    .get("availability_issues")
                    .filter(|v| v.as_array().is_some_and(|v| !v.is_empty()))
                    .ok_or("Unavailable evidence must have availability issues")?;
                out.missing.push(json!({"name":name,"reasons":reasons,"availability_issues":issues,"span":item["span"]}));
            }
        }
        if let Some(basis) = source.get("structural_basis").filter(|v| !v.is_null()) {
            let atom = basis["atom"]
                .as_str()
                .ok_or("Source map structural basis must name an unmapped axiom")?;
            if !premises.get(atom).is_some_and(|p| p["kind"] == "axiom")
                || out.atoms.contains_key(atom)
            {
                return Err("Source map structural basis must name an unmapped axiom".into());
            }
            out.atoms.insert(atom.into(),json!({"kind":"structural basis","name":"compiler_basis","meaning":basis["meaning"]}));
        }
        let contrary_pairs = theory["contraries"]
            .as_array()
            .expect("validated contraries")
            .iter()
            .map(|v| {
                (
                    v["attacker"].as_str().expect("attacker"),
                    v["target"].as_str().expect("target"),
                )
            })
            .collect::<BTreeSet<_>>();
        if let Some(contraries) = source["contraries"].as_array() {
            for item in contraries {
                let attacker = item["attacker"].as_str().expect("source schema attacker");
                let target = item["target"].as_str().expect("source schema target");
                if !contrary_pairs.contains(&(attacker, target)) {
                    return Err("Source map contrary is absent from the formal theory".into());
                }
                out.contraries.insert(
                    (attacker.into(), target.into()),
                    json!({"objection":item["objection"],"annotation":item["annotation"]}),
                );
            }
        }
        Ok(out)
    }
}

fn entries<'a>(
    source: &'a Value,
    kind: &str,
) -> Result<impl Iterator<Item = (&'a String, &'a Value)>, String> {
    static EMPTY: std::sync::OnceLock<serde_json::Map<String, Value>> = std::sync::OnceLock::new();
    let values = match source.get(kind) {
        Some(value) => value
            .as_object()
            .ok_or_else(|| format!("Source map {kind} must contain named declarations"))?,
        None => EMPTY.get_or_init(serde_json::Map::new),
    };
    Ok(values.iter())
}

// Source correspondence uses Python equality, including bool/integer equality
// for the deliberately open source-map rank field. Formal identity stays exact.
fn python_equal(left: &Value, right: &Value) -> bool {
    let numeric = |value: &Value| match value {
        Value::Bool(v) => json!(u8::from(*v)),
        v => v.clone(),
    };
    left == right
        || numbers::compare(&numeric(left), &numeric(right)) == Some(std::cmp::Ordering::Equal)
}
