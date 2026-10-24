//! Bounded projections of stored assessments; raw observations never enter a packet.
use eal_core::methods::MethodRegistry;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use std::collections::{BTreeSet, VecDeque};

mod graph;
mod projection;
pub(crate) mod shape;
use shape::{bounded, digest, identifier, ids, mapping, record_identifier, scalar, state};

pub const PACKET_SCHEMA: &str = "EAL/assessment-packet/2";
/// Version 3 adds mandatory qualification for computationally incomplete results.
pub const INCOMPLETE_PACKET_SCHEMA: &str = "EAL/assessment-packet/3";
const CLAIM_STATES: &[&str] = &["supported", "contested", "unsupported", "out_of_scope"];

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct PacketLimits {
    pub max_bytes: usize,
    pub max_claims: usize,
    pub max_arguments: usize,
    pub max_objections: usize,
    pub max_assumptions: usize,
    pub max_evidence: usize,
    pub max_outputs: usize,
    pub max_availability_issues: usize,
    pub max_observations_per_claim: usize,
    pub max_statement_bytes: usize,
}

impl Default for PacketLimits {
    fn default() -> Self {
        Self {
            max_bytes: 16_384,
            max_claims: 16,
            max_arguments: 24,
            max_objections: 32,
            max_assumptions: 32,
            max_evidence: 48,
            max_outputs: 16,
            max_availability_issues: 6,
            max_observations_per_claim: 16,
            max_statement_bytes: 1024,
        }
    }
}

impl PacketLimits {
    pub fn validate(&self) -> Result<(), String> {
        if self.max_bytes < 4096 {
            return Err("Packet byte limit must be at least 4096".into());
        }
        if !(1..=4096).contains(&self.max_statement_bytes) {
            return Err("Statement byte limit must be an integer between 1 and 4096".into());
        }
        for (name, value) in [
            ("max_claims", self.max_claims),
            ("max_arguments", self.max_arguments),
            ("max_objections", self.max_objections),
            ("max_assumptions", self.max_assumptions),
            ("max_evidence", self.max_evidence),
            ("max_outputs", self.max_outputs),
            ("max_availability_issues", self.max_availability_issues),
            (
                "max_observations_per_claim",
                self.max_observations_per_claim,
            ),
        ] {
            if !(1..=128).contains(&value) {
                return Err(format!("{name} must be an integer between 1 and 128"));
            }
        }
        Ok(())
    }
}

#[derive(Clone, Default)]
pub struct AssessmentPacketBuilder {
    limits: PacketLimits,
    method_registry: Option<MethodRegistry>,
}

impl AssessmentPacketBuilder {
    pub fn new(
        limits: PacketLimits,
        method_registry: Option<MethodRegistry>,
    ) -> Result<Self, String> {
        limits.validate()?;
        Ok(Self {
            limits,
            method_registry,
        })
    }

    pub fn build(
        &self,
        assessment: &Value,
        claims: Option<&[String]>,
        collection: Option<&Value>,
    ) -> Result<Value, String> {
        self.limits.validate()?;
        shape::preflight(assessment)?;
        if let Some(collection) = collection {
            shape::preflight(collection)?;
        }
        if !assessment.is_object() || assessment["valid"] != true {
            return Err("A packet requires a valid assessment".into());
        }
        let complete = match assessment.get("complete") {
            None => true,
            Some(Value::Bool(complete)) => *complete,
            Some(_) => return Err("Assessment completeness must be a boolean".into()),
        };
        if complete
            && assessment
                .get("arguments")
                .and_then(Value::as_object)
                .is_some_and(|arguments| {
                    arguments.values().any(|argument| {
                        argument["reasoning_result"]["status"]
                            == eal_core::methods::MethodResultStatus::Incomplete.as_str()
                    })
                })
        {
            return Err("Assessment completeness contradicts an incomplete method result".into());
        }
        let assessment_id = assessment["assessment_id"]
            .as_str()
            .filter(|s| {
                !s.is_empty() && s.len() <= 128 && s.bytes().all(|b| (33..=126).contains(&b))
            })
            .ok_or("A packet requires a stored assessment identifier")?;
        let entries = assessment["claims"]
            .as_object()
            .ok_or("A packet requires assessed claims")?;
        let selected: Vec<String> = match claims {
            None => {
                let mut names = entries
                    .keys()
                    .map(|s| identifier(&Value::String(s.clone())))
                    .collect::<Result<Vec<_>, _>>()?;
                names.sort();
                names.truncate(self.limits.max_claims);
                names
            }
            Some(names) => {
                if names.is_empty()
                    || names.len() > self.limits.max_claims
                    || names.iter().collect::<BTreeSet<_>>().len() != names.len()
                {
                    return Err("Select one to the configured maximum of distinct claims".into());
                }
                let names = names
                    .iter()
                    .map(|s| identifier(&Value::String(s.clone())))
                    .collect::<Result<Vec<_>, _>>()?;
                if names.iter().any(|s| !entries.contains_key(s)) {
                    return Err("The requested claim is absent from the assessment".into());
                }
                names
            }
        };
        let collection_records = projection::collection_records(assessment, collection)?;
        let collection_id = &assessment["collection_id"];
        if !collection_id.is_null()
            && record_identifier(collection_id).as_ref() != Some(collection_id)
        {
            return Err("Collection identifier must be bounded".into());
        }
        let mut packet = json!({"schema": if complete {PACKET_SCHEMA} else {INCOMPLETE_PACKET_SCHEMA}, "assessment_id": assessment_id,
            "collection_id": collection_id, "source_digest": digest(&assessment["source_digest"]),
            "context_fingerprint": digest(&assessment["context_fingerprint"]),
            "method_registry_fingerprint": digest(&assessment["method_registry_fingerprint"]),
            "claims": {}, "premise_claims": {}, "arguments": {}, "assumptions": {}, "objections": {}, "evidence": {},
            "omitted": {}, "summary_complete": true,
            "full_explanation": {"operation": "explain", "assessment_id": assessment_id, "detail": "full"}});
        if !complete {
            packet["complete"] = json!(false);
        }
        let mut omitted = Map::new();
        if claims.is_none() && entries.len() > selected.len() {
            add(&mut omitted, "claims", entries.len() - selected.len());
        }
        let empty = Map::new();
        let arguments = group(assessment, "arguments", &empty)?;
        let objections = group(assessment, "objections", &empty)?;
        let assumptions = group(assessment, "assumptions", &empty)?;
        let evidence = group(assessment, "evidence", &empty)?;
        let reasoning = group(assessment, "reasoning", &empty)?;
        let mut argument_ids = BTreeSet::new();
        let mut objection_ids = BTreeSet::new();
        for name in &selected {
            let entry = entries[name]
                .as_object()
                .ok_or("Claim assessment must be an object")?;
            let support = ids(entry.get("supporting_arguments"), false)?;
            let contested = ids(entry.get("contested_arguments"), false)?;
            let mut related = Vec::new();
            for (id, item) in arguments {
                if item.is_object() && item["conclusion"] == *name {
                    related.push(identifier(&Value::String(id.clone()))?);
                }
            }
            related.sort();
            argument_ids.extend(related.iter().cloned());
            let active = ids(entry.get("objections"), false)?;
            let undecided = ids(entry.get("undecided_objections"), false)?;
            objection_ids.extend(active.iter().chain(undecided.iter()).cloned());
            for (id, item) in objections {
                if item.is_object() && item["target_kind"] == "claim" && item["target"] == *name {
                    objection_ids.insert(identifier(&Value::String(id.clone()))?);
                }
            }
            let mut claim = json!({"status": state(entry.get("status"), CLAIM_STATES)?,
                "grounded_label": bounded(entry.get("grounded_label")),
                "supporting_arguments": prefix(support, self.limits.max_arguments),
                "contested_arguments": prefix(contested, self.limits.max_arguments),
                "related_arguments": prefix(related, self.limits.max_arguments),
                "active_objections": prefix(active, self.limits.max_objections),
                "undecided_objections": prefix(undecided, self.limits.max_objections)});
            if !complete {
                claim["assessment_complete"] = json!(false);
            }
            claim
                .as_object_mut()
                .expect("constructed object")
                .extend(self.claim_meaning(entry, &mut omitted));
            if let Some(proposition) = entry.get("proposition").and_then(Value::as_object)
                && let Some(comparison) = proposition.get("result").and_then(Value::as_object)
                && comparison.get("expected").is_some_and(scalar)
            {
                claim["typed_result"] = json!({"path": bounded(comparison.get("path")),
                    "operator": comparison.get("operator").filter(|v| v.as_str().is_some_and(|s| ["==", "!=", "<", "<=", ">", ">="].contains(&s))),
                    "expected": comparison["expected"], "unit": bounded(proposition.get("unit"))});
            }
            packet["claims"][name] = claim;
        }
        let by_conclusion = graph::by_conclusion(arguments)?;
        let mut pending: Vec<_> = argument_ids.iter().cloned().collect();
        let mut visited = BTreeSet::new();
        let mut premise_ids = BTreeSet::new();
        while !pending.is_empty() && visited.len() < 128 {
            let id = pending.pop().expect("nonempty");
            if visited.contains(&id) || !arguments.contains_key(&id) {
                continue;
            }
            visited.insert(id.clone());
            let entry = arguments[&id]
                .as_object()
                .ok_or("Argument assessment must be an object")?;
            let Some(dependencies) = mapping(entry.get("dependencies")) else {
                continue;
            };
            for premise in ids(dependencies.get("premises"), false)? {
                premise_ids.insert(premise.clone());
                for alternative in by_conclusion.get(&premise).into_iter().flatten() {
                    if argument_ids.insert(alternative.clone()) {
                        pending.push(alternative.clone());
                    }
                }
            }
        }
        if !pending.is_empty() {
            add(&mut omitted, "premise_traversal", pending.len());
        }
        let mut assumption_ids = BTreeSet::new();
        let mut evidence_ids = BTreeSet::new();
        for name in &argument_ids {
            let entry = arguments
                .get(name)
                .and_then(Value::as_object)
                .ok_or("Argument assessment must be an object")?;
            let dependencies = mapping(entry.get("dependencies")).unwrap_or(&empty);
            assumption_ids.extend(ids(dependencies.get("assumptions"), false)?);
            evidence_ids.extend(ids(dependencies.get("evidence"), false)?);
            objection_ids.extend(ids(entry.get("attackers"), true)?);
            let rule = dependencies.get("reasoning");
            if let Some(item) = rule
                .and_then(Value::as_str)
                .and_then(|id| reasoning.get(id))
                .and_then(Value::as_object)
            {
                evidence_ids.extend(ids(item.get("backing"), false)?);
            }
            for (id, item) in objections {
                if item.is_object()
                    && item["target_kind"] == "reasoning"
                    && item.get("target") == rule
                {
                    objection_ids.insert(identifier(&Value::String(id.clone()))?);
                }
            }
        }
        for name in &assumption_ids {
            if let Some(item) = assumptions.get(name).and_then(Value::as_object) {
                if let Some(validation) = item.get("validation").filter(|v| v.is_string()) {
                    evidence_ids.insert(identifier(validation)?);
                }
                if let Some(statuses) = item.get("objection_statuses") {
                    for id in statuses
                        .as_object()
                        .ok_or("Assumption objection statuses must be a mapping")?
                        .keys()
                    {
                        objection_ids.insert(identifier(&Value::String(id.clone()))?);
                    }
                }
            }
        }
        let mut queue: VecDeque<_> = objection_ids.iter().cloned().collect();
        while !queue.is_empty() && objection_ids.len() <= self.limits.max_objections {
            let current = queue.pop_front().expect("nonempty");
            let Some(item) = objections.get(&current).and_then(Value::as_object) else {
                continue;
            };
            for candidate in ids(item.get("attackers"), true)? {
                if objection_ids.insert(candidate.clone()) {
                    queue.push_back(candidate);
                }
            }
            evidence_ids.extend(ids(item.get("evidence"), false)?);
            premise_ids.extend(ids(item.get("premises"), false)?);
        }
        for name in premise_ids.iter().take(self.limits.max_claims) {
            if let Some(item) = entries.get(name).and_then(Value::as_object) {
                let mut projected = self.claim_meaning(item, &mut omitted);
                projected.insert(
                    "status".into(),
                    json!(state(item.get("status"), CLAIM_STATES)?),
                );
                packet["premise_claims"][name] = Value::Object(projected);
            }
        }
        if premise_ids.len() > self.limits.max_claims {
            add(
                &mut omitted,
                "premise_claims",
                premise_ids.len() - self.limits.max_claims,
            );
        }
        let objection_routes: BTreeSet<_> = premise_ids
            .iter()
            .flat_map(|p| by_conclusion.get(p).into_iter().flatten())
            .filter(|id| !argument_ids.contains(*id))
            .collect();
        if !objection_routes.is_empty() {
            add(
                &mut omitted,
                "objection_premise_routes",
                objection_routes.len(),
            );
        }
        for name in &selected {
            let (references, missed) = graph::claim_observations(
                name,
                entries,
                arguments,
                assumptions,
                objections,
                reasoning,
                evidence,
            )?;
            evidence_ids.extend(
                references
                    .iter()
                    .filter_map(|r| r["evidence_id"].as_str())
                    .map(str::to_owned),
            );
            if missed {
                add(&mut omitted, "claim_dependency_routes", 1);
            }
            if references.len() > self.limits.max_observations_per_claim {
                add(
                    &mut omitted,
                    "claim_observations",
                    references.len() - self.limits.max_observations_per_claim,
                );
            }
            packet["claims"][name]["observation_ids"] =
                json!(prefix(references, self.limits.max_observations_per_claim));
        }
        let mut sorted_arguments: Vec<_> = argument_ids.into_iter().collect();
        sorted_arguments.sort_by_key(|id| {
            (
                !arguments.get(id).is_some_and(|v| {
                    ["supported", "contested"].contains(&v["status"].as_str().unwrap_or(""))
                }),
                id.clone(),
            )
        });
        self.project(
            &mut packet,
            sorted_arguments,
            arguments,
            self.limits.max_arguments,
            "arguments",
            &mut omitted,
        )?;
        let mut sorted_objections: Vec<_> = objection_ids.into_iter().collect();
        sorted_objections.sort_by_key(|id| {
            (
                ["active", "undecided", "defeated", "inactive"]
                    .iter()
                    .position(|s| objections.get(id).is_some_and(|v| v["status"] == *s))
                    .unwrap_or(4),
                id.clone(),
            )
        });
        self.project(
            &mut packet,
            sorted_objections,
            objections,
            self.limits.max_objections,
            "objections",
            &mut omitted,
        )?;
        self.project(
            &mut packet,
            assumption_ids.into_iter().collect(),
            assumptions,
            self.limits.max_assumptions,
            "assumptions",
            &mut omitted,
        )?;
        self.project(
            &mut packet,
            evidence_ids.into_iter().collect(),
            evidence,
            self.limits.max_evidence,
            "evidence",
            &mut omitted,
        )?;
        for (name, item) in packet["evidence"].as_object_mut().expect("constructed map") {
            let projected = item.as_object_mut().expect("projected object");
            for (temporary, label) in [
                ("issues_omitted", "availability_issues"),
                ("predicate_failures_omitted", "predicate_failures"),
            ] {
                if let Some(missed) = projected.remove(temporary).and_then(|v| v.as_u64())
                    && missed > 0
                {
                    add(&mut omitted, label, missed as usize);
                }
            }
            if let Some(record) = collection_records.and_then(|r| r.get(name)) {
                projection::observation_time(projected, assessment, &evidence[name], record)?;
            }
        }
        packet["summary_complete"] = json!(omitted.is_empty());
        packet["omitted"] = Value::Object(omitted);
        if shape::size(&packet)? > self.limits.max_bytes {
            let claims = packet["claims"].as_object().expect("constructed map").iter().map(|(name, item)| (name.clone(), if complete { json!({"status": item["status"], "statement": null, "prose_verified": false}) } else { json!({"status": item["status"], "statement": null, "prose_verified": false, "assessment_complete": false}) })).collect();
            let mut fallback = Map::new();
            for key in [
                "schema",
                "assessment_id",
                "collection_id",
                "source_digest",
                "context_fingerprint",
                "method_registry_fingerprint",
                "full_explanation",
            ] {
                fallback.insert(key.into(), packet[key].take());
            }
            if !complete {
                fallback.insert("complete".into(), json!(false));
            }
            fallback.insert("claims".into(), Value::Object(claims));
            fallback.insert("summary_complete".into(), json!(false));
            fallback.insert("omitted".into(), json!({"details": "packet_byte_limit"}));
            packet = Value::Object(fallback);
            if shape::size(&packet)? > self.limits.max_bytes {
                return Err(
                    "Requested claims exceed the packet byte limit; request fewer claims".into(),
                );
            }
        }
        Ok(packet)
    }

    fn claim_meaning(
        &self,
        item: &Map<String, Value>,
        omitted: &mut Map<String, Value>,
    ) -> Map<String, Value> {
        let mut result = json!({"statement": null, "prose_verified": false})
            .as_object()
            .expect("object")
            .clone();
        let Some(statement) = item.get("statement").and_then(Value::as_str) else {
            add(omitted, "claim_statements", 1);
            return result;
        };
        let mut end = statement.len().min(self.limits.max_statement_bytes);
        while !statement.is_char_boundary(end) {
            end -= 1;
        }
        result.insert("statement".into(), json!(&statement[..end]));
        result.insert(
            "prose_verified".into(),
            json!(item.get("prose_verified") == Some(&Value::Bool(true))),
        );
        if end < statement.len() {
            result.insert("statement_truncated".into(), json!(true));
            result.insert("prose_verified".into(), json!(false));
            add(omitted, "claim_statement_bytes", statement.len() - end);
        }
        result
    }

    fn project(
        &self,
        packet: &mut Value,
        ids: Vec<String>,
        entries: &Map<String, Value>,
        limit: usize,
        label: &str,
        omitted: &mut Map<String, Value>,
    ) -> Result<(), String> {
        for name in ids.iter().take(limit) {
            if let Some(item) = entries.get(name).and_then(Value::as_object) {
                packet[label][name] = match label {
                    "arguments" => self.argument(item)?,
                    "objections" => projection::objection(item)?,
                    "assumptions" => projection::assumption(item)?,
                    "evidence" => self.evidence(item)?,
                    _ => unreachable!(),
                };
            }
        }
        if ids.len() > limit {
            add(omitted, label, ids.len() - limit);
        }
        Ok(())
    }
}

fn group<'a>(
    assessment: &'a Value,
    key: &str,
    empty: &'a Map<String, Value>,
) -> Result<&'a Map<String, Value>, String> {
    match assessment.get(key) {
        None => Ok(empty),
        Some(value) => value
            .as_object()
            .ok_or_else(|| "Assessment dependencies must be mappings".into()),
    }
}
fn prefix<T>(mut values: Vec<T>, limit: usize) -> Vec<T> {
    values.truncate(limit);
    values
}
fn add(omitted: &mut Map<String, Value>, key: &str, amount: usize) {
    let old = omitted.get(key).and_then(Value::as_u64).unwrap_or(0);
    omitted.insert(key.into(), json!(old + amount as u64));
}
