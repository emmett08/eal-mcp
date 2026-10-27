//! Assess local method applications before their support/attack labels are decided.
mod input;
use super::{name_repr, predicate, status};
use crate::{
    canonical,
    methods::{self, MethodRegistry},
    propositions,
    semantic::{Argument, Claim, Program},
};
use indexmap::IndexMap;
use serde_json::{Value, json};
use std::collections::BTreeSet;

pub(crate) struct SourceAssessment {
    pub by_conclusion: IndexMap<String, Vec<String>>,
    pub source_claims: IndexMap<String, Value>,
    pub source_arguments: IndexMap<String, bool>,
    pub local_arguments: IndexMap<String, bool>,
}

pub(crate) fn assess_source_claims(
    program: &Program,
    records: &Value,
    result: &mut Value,
    registry: &MethodRegistry,
) -> Result<SourceAssessment, String> {
    let mut by_conclusion: IndexMap<String, Vec<String>> = program
        .claims
        .keys()
        .map(|name| (name.clone(), vec![]))
        .collect();
    for argument in program.arguments.values() {
        by_conclusion
            .get_mut(&argument.conclusion)
            .ok_or("Argument conclusion is undeclared")?
            .push(argument.name.clone());
    }
    let mut state = SourceState {
        program,
        records,
        result,
        registry,
        sources: SourceAssessment {
            by_conclusion,
            source_claims: IndexMap::new(),
            source_arguments: IndexMap::new(),
            local_arguments: IndexMap::new(),
        },
        visiting: BTreeSet::new(),
    };
    for name in program.claims.keys() {
        state.claim(name)?;
    }
    Ok(state.sources)
}
struct SourceState<'a> {
    program: &'a Program,
    records: &'a Value,
    result: &'a mut Value,
    registry: &'a MethodRegistry,
    sources: SourceAssessment,
    visiting: BTreeSet<String>,
}
impl SourceState<'_> {
    fn claim(&mut self, name: &str) -> Result<Value, String> {
        if let Some(known) = self.sources.source_claims.get(name) {
            return Ok(known.clone());
        }
        if !self.visiting.insert(name.to_owned()) {
            return Err("Cyclic premise dependency reached evaluation".into());
        }
        if self.visiting.len().saturating_sub(1) > self.program.limits.premise_depth {
            return Err("Premise depth budget exceeded".into());
        }
        let claim = &self.program.claims[name];
        let in_scope = status(self.result, "environments", &claim.environment) == "matched";
        let argument_names = self.sources.by_conclusion[name].clone();
        let mut viable = vec![];
        for argument_name in argument_names {
            let argument = self.program.arguments[&argument_name].clone();
            let method = &self.program.reasoning[&argument.reasoning];
            let mut reasons = vec![];
            let mut computation = json!({"status":methods::MethodResultStatus::NotRun,"method":method.method,"reasons":["Required sources are unavailable or out of scope"],"details":{}});
            let dependencies = json!({"evidence":argument.evidence,"assumptions":argument.assumptions,"premises":argument.premises,"reasoning":argument.reasoning});
            let mut premise_entries = vec![];
            for premise in &argument.premises {
                let mut entry = self.claim(premise)?;
                entry["id"] = json!(premise);
                premise_entries.push(entry);
            }
            let source_available = argument
                .evidence
                .iter()
                .all(|e| status(self.result, "evidence", e) == "available")
                && argument
                    .assumptions
                    .iter()
                    .all(|a| status(self.result, "assumptions", a) == "supported")
                && status(self.result, "reasoning", &argument.reasoning) == "supported";
            for evidence in &argument.evidence {
                reasons.push(format!(
                    "Evidence {} is {}",
                    name_repr(evidence),
                    status(self.result, "evidence", evidence)
                ));
            }
            for assumption in &argument.assumptions {
                reasons.push(format!(
                    "Assumption {} is locally {}",
                    name_repr(assumption),
                    status(self.result, "assumptions", assumption)
                ));
            }
            for premise in &premise_entries {
                reasons.push(format!(
                    "Premise claim {} {} a source-usable derivation",
                    name_repr(premise["id"].as_str().unwrap()),
                    if premise["source_usable"] == true {
                        "has"
                    } else {
                        "lacks"
                    }
                ));
            }
            let (mut state, usable) = if !in_scope {
                reasons.push(format!(
                    "Conclusion environment {} does not match supplied context",
                    name_repr(&claim.environment)
                ));
                ("out_of_scope", false)
            } else if !source_available {
                ("unsupported", false)
            } else {
                let method_premises: Vec<_> = premise_entries
                    .iter()
                    .map(|p| {
                        let mut p = p.clone();
                        p["status"] = json!("supported");
                        p
                    })
                    .collect();
                let assessed = compute_argument(
                    self.program,
                    &argument,
                    claim,
                    self.records,
                    &method_premises,
                    self.registry,
                )?;
                computation = assessed.0;
                computation["premise_acceptance"] = json!("deferred_to_composed_solver");
                reasons.extend(
                    computation["reasons"]
                        .as_array()
                        .unwrap_or(&vec![])
                        .iter()
                        .filter_map(Value::as_str)
                        .map(str::to_owned),
                );
                if let Some(predicates) = computation["predicates"].as_array() {
                    reasons.extend(
                        predicates
                            .iter()
                            .filter_map(|p| p["reason"].as_str())
                            .map(str::to_owned),
                    );
                }
                if let Some(binding) = computation.get("binding")
                    && let Some(binding_reasons) = binding["reasons"].as_array()
                {
                    reasons.extend(
                        binding_reasons
                            .iter()
                            .filter_map(Value::as_str)
                            .map(str::to_owned),
                    );
                }
                (
                    if assessed.1 {
                        "supported"
                    } else {
                        "unsupported"
                    },
                    assessed.1,
                )
            };
            self.sources
                .local_arguments
                .insert(argument_name.clone(), usable);
            let possible = usable && premise_entries.iter().all(|p| p["source_usable"] == true);
            self.sources
                .source_arguments
                .insert(argument_name.clone(), possible);
            if state == "supported" && !possible {
                state = "unsupported";
            }
            self.result["arguments"][&argument_name] = json!({"status":state,"reasons":reasons,"conclusion":name,"dependencies":dependencies,"reasoning_result":computation,"source_usable":possible,"locally_usable":usable,"origin":argument.origin});
            if possible {
                viable.push(argument_name);
            }
        }
        let entry = json!({"status":if viable.is_empty(){"unsupported"}else{"supported"},"source_usable":!viable.is_empty(),"statement":claim.statement,"environment":claim.environment,"proposition":super::proposition_value(claim.proposition.as_ref()),"source_arguments":viable});
        self.sources
            .source_claims
            .insert(name.to_owned(), entry.clone());
        self.visiting.remove(name);
        Ok(entry)
    }
}

fn compute_argument(
    program: &Program,
    argument: &Argument,
    claim: &Claim,
    records: &Value,
    premises: &[Value],
    registry: &MethodRegistry,
) -> Result<(Value, bool), String> {
    let method = &program.reasoning[&argument.reasoning];
    if method.transfer.is_some() {
        let mut outcome = crate::scope_transfer::computation(program, argument);
        let checks: Vec<_> = method
            .predicates
            .iter()
            .map(|p| predicate(p, &outcome["details"], &program.limits))
            .collect();
        let usable = checks.iter().all(|(ok, _)| *ok);
        outcome["predicates"] = json!(
            checks
                .into_iter()
                .map(|(ok, reason)| json!({"holds":ok,"reason":reason}))
                .collect::<Vec<_>>()
        );
        return Ok((outcome, usable));
    }
    let mut source_ids: BTreeSet<_> = argument
        .evidence
        .iter()
        .chain(method.backing.iter())
        .cloned()
        .collect();
    source_ids.extend(
        argument
            .assumptions
            .iter()
            .map(|a| program.assumptions[a].validation.clone()),
    );
    let mut sources: Vec<_> = source_ids
        .into_iter()
        .map(|id| json!({"id":id,"kind":program.evidence[&id].kind,"value":records[&id]["value"]}))
        .collect();
    let mut binding = None;
    let mut bound_observation_digest = None;
    let mut authored_input = None;
    if let Some(proposition) = &claim.proposition {
        let selected = argument
            .binding
            .as_deref()
            .and_then(|id| sources.iter().find(|s| s["id"] == id));
        if let Some(selected) = selected {
            bound_observation_digest = Some(canonical::canonical_digest(
                &selected["value"],
                &program.limits,
            )?);
        }
        let (payload, trace) = input::prepare(
            proposition,
            &method.method,
            selected.map(|s| (s["id"].as_str().expect("source id"), &s["value"])),
            registry,
            &program.limits,
        );
        if let Some(payload) = payload {
            if let Some(id) = &argument.binding {
                for source in &mut sources {
                    if source["id"] == *id {
                        source["value"] = payload.clone();
                    }
                }
            } else {
                authored_input = Some(payload);
            }
        }
        binding = Some(trace);
    }
    if let Some(trace) = &binding
        && trace["status"] == "unsupported"
    {
        return Ok((
            json!({"status":"unsupported","method":method.method,"reasons":trace["reasons"],"details":{},"binding":trace}),
            false,
        ));
    }
    let mut computation = if let Some(payload) = &authored_input {
        methods::assess_input_with_limits(&method.method, payload, registry, &program.limits)
    } else {
        methods::assess_mode_with_limits(
            &method.method,
            &sources,
            premises,
            registry,
            &program.limits,
        )
    };
    if let Some(trace) = binding.as_mut() {
        if let Some(digest) = &bound_observation_digest {
            computation["bound_observation_digest"] = json!(digest);
        }
        if computation["status"] == "supported" {
            *trace = propositions::check_result_with_limits(
                claim.proposition.as_ref().unwrap(),
                &method.method,
                &computation["details"],
                trace,
                registry,
                &program.limits,
            );
        } else {
            trace["status"] = json!("unsupported");
            trace["reasons"] = json!(["The bound computation did not produce a usable result"]);
        }
        computation["binding"] = trace.clone();
    }
    if computation["status"] != "supported" {
        return Ok((computation, false));
    }
    let checks: Vec<_> = method
        .predicates
        .iter()
        .map(|p| predicate(p, &computation["details"], &program.limits))
        .collect();
    let usable = checks.iter().all(|(ok, _)| *ok)
        && binding
            .as_ref()
            .is_none_or(|trace| trace["status"] == "supported");
    computation["predicates"] = json!(
        checks
            .into_iter()
            .map(|(ok, reason)| json!({"holds":ok,"reason":reason}))
            .collect::<Vec<_>>()
    );
    Ok((computation, usable))
}
