//! Dependency kinds, environmental scope and explicit computational bindings.
use super::{Checker, check_json_resources, objection_scopes, parse_time};
use std::collections::BTreeSet;

impl Checker<'_> {
    pub(super) fn tools_and_evidence(&mut self) {
        let p = self.program;
        for value in p.tools.values() {
            if value.version.trim().is_empty() {
                self.error(
                    "empty_version",
                    "A tool requires an explicit nonempty version",
                    Some(&value.name),
                );
            }
        }
        for value in p.evidence.values() {
            self.identifier(&value.kind, &value.name);
            if let Some(tool) = &value.tool {
                self.reference(tool, p.tools.contains_key(tool), "tool", &value.name);
            }
            self.reference(
                &value.environment,
                p.environments.contains_key(&value.environment),
                "environment",
                &value.name,
            );
            if !value.max_age.is_finite() || value.max_age < 0.0 {
                self.error(
                    "invalid_age",
                    "max_age must be a finite nonnegative number of seconds",
                    Some(&value.name),
                );
            }
            if check_json_resources(&value.input, &p.limits).is_err() {
                self.error(
                    "invalid_input",
                    "Tool input must be finite JSON data",
                    Some(&value.name),
                );
            }
        }
    }

    pub(super) fn assumptions(&mut self) {
        let p = self.program;
        for value in p.assumptions.values() {
            if value.statement.trim().is_empty() {
                self.error(
                    "empty_statement",
                    "An assumption requires a statement",
                    Some(&value.name),
                );
            }
            self.reference(
                &value.environment,
                p.environments.contains_key(&value.environment),
                "environment",
                &value.name,
            );
            if self.reference(
                &value.validation,
                p.evidence.contains_key(&value.validation),
                "validation evidence",
                &value.name,
            ) {
                self.scope(
                    &p.evidence[&value.validation].environment,
                    &value.environment,
                    &value.name,
                    &value.validation,
                );
            }
            let mut dates = Vec::new();
            for (label, timestamp) in [
                ("valid_from", &value.valid_from),
                ("valid_until", &value.valid_until),
            ] {
                if let Some(timestamp) = timestamp {
                    match parse_time(timestamp) {
                        Ok(date) => dates.push(date),
                        Err(_) => self.error(
                            "invalid_timestamp",
                            format!("{label} requires an ISO-8601 timestamp with timezone"),
                            Some(&value.name),
                        ),
                    }
                }
            }
            if dates.len() == 2 && dates[0] >= dates[1] {
                self.error(
                    "invalid_interval",
                    "valid_from must precede valid_until",
                    Some(&value.name),
                );
            }
        }
    }

    pub(super) fn arguments(&mut self) {
        let p = self.program;
        for value in p.arguments.values() {
            let has_claim = self.reference(
                &value.conclusion,
                p.claims.contains_key(&value.conclusion),
                "conclusion claim",
                &value.name,
            );
            let has_reasoning = self.reference(
                &value.reasoning,
                p.reasoning.contains_key(&value.reasoning),
                "reasoning",
                &value.name,
            );
            if value.evidence.is_empty()
                && value.assumptions.is_empty()
                && value.premises.is_empty()
            {
                self.error(
                    "empty_argument",
                    "An argument requires evidence, assumptions or premise claims",
                    Some(&value.name),
                );
            }
            let expected = if has_claim {
                Some(p.claims[&value.conclusion].environment.as_str())
            } else {
                None
            };
            let relation = if has_reasoning {
                p.reasoning[&value.reasoning].transfer.as_ref()
            } else {
                None
            };
            for item in &value.evidence {
                if self.reference(item, p.evidence.contains_key(item), "evidence", &value.name)
                    && let Some(expected) = expected
                {
                    self.scope(&p.evidence[item].environment, expected, &value.name, item);
                }
            }
            for item in &value.assumptions {
                if self.reference(
                    item,
                    p.assumptions.contains_key(item),
                    "assumption",
                    &value.name,
                ) && let Some(expected) = expected
                {
                    self.scope(
                        &p.assumptions[item].environment,
                        expected,
                        &value.name,
                        item,
                    );
                }
            }
            for item in &value.premises {
                if self.reference(
                    item,
                    p.claims.contains_key(item),
                    "premise claim",
                    &value.name,
                ) && let Some(expected) = expected
                {
                    self.scope(
                        &p.claims[item].environment,
                        relation.map(|r| r.source.as_str()).unwrap_or(expected),
                        &value.name,
                        item,
                    );
                }
            }
            if !has_reasoning {
                continue;
            }
            let method = &p.reasoning[&value.reasoning];
            let contract = self.registry.get(&method.method);
            let mut source_ids: BTreeSet<_> = value
                .evidence
                .iter()
                .chain(method.backing.iter())
                .cloned()
                .collect();
            for name in &value.assumptions {
                if let Some(assumption) = p.assumptions.get(name) {
                    source_ids.insert(assumption.validation.clone());
                }
            }
            let proposition = if has_claim {
                p.claims[&value.conclusion].proposition.as_ref()
            } else {
                None
            };
            let authored_query = p.profile.as_deref() == Some("execution/1")
                && proposition.is_some()
                && value.binding.is_none();
            if relation.is_some() {
                for message in crate::scope_transfer::transfer_errors(p, value) {
                    self.error("invalid_scope_transfer", message, Some(&value.name));
                }
            } else if let Some(proposition) = proposition {
                if let Some(binding) = &value.binding {
                    if self.reference(
                        binding,
                        p.evidence.contains_key(binding),
                        "bound evidence",
                        &value.name,
                    ) {
                        if !source_ids.contains(binding) {
                            self.error(
                                "binding_source",
                                "Bound evidence must be a source of this argument",
                                Some(&value.name),
                            );
                        }
                        if contract.and_then(|c| c.evidence_kind.as_deref())
                            != Some(p.evidence[binding].kind.as_str())
                        {
                            self.error(
                                "binding_source",
                                "Binding must select the method's computational evidence",
                                Some(&value.name),
                            );
                        }
                    }
                } else if authored_query {
                    if let Some(contract) = contract {
                        for message in crate::methods::schema_errors_with_limits(
                            &proposition.query,
                            &contract.input_schema,
                            "query",
                            &p.limits,
                        ) {
                            self.error("authored_method_input", message, Some(&value.name));
                        }
                    }
                } else {
                    self.error(
                        "missing_binding",
                        "A typed conclusion requires an explicit evidence binding",
                        Some(&value.name),
                    );
                }
                for message in crate::propositions::proposition_errors_with_limits(
                    proposition,
                    Some(&method.method),
                    self.registry,
                    &p.limits,
                ) {
                    self.error("proposition_method", message, Some(&value.name));
                }
            } else if value.binding.is_some() {
                self.error(
                    "untyped_binding",
                    "An evidence binding requires a typed conclusion",
                    Some(&value.name),
                );
            }
            if proposition.is_none()
                && contract.is_some()
                && method.method != "structured/1"
                && method.predicates.is_empty()
            {
                self.error(
                    "missing_reasoning_predicate",
                    "An untyped computational conclusion requires an explicit output predicate",
                    Some(&value.name),
                );
            }
            let kinds: Vec<_> = source_ids
                .iter()
                .filter_map(|id| p.evidence.get(id).map(|e| e.kind.clone()))
                .collect();
            let mode_errors = if authored_query {
                if contract.is_none() {
                    vec![format!(
                        "Unknown registered reasoning method {:?}",
                        method.method
                    )]
                } else {
                    Vec::new()
                }
            } else {
                crate::methods::validate_mode_with_limits(
                    &method.method,
                    &kinds,
                    self.registry,
                    &p.limits,
                )
            };
            for message in mode_errors {
                self.error("reasoning_evidence_contract", message, Some(&value.name));
            }
            if let Some(expected) = expected {
                for item in &method.backing {
                    if let Some(evidence) = p.evidence.get(item) {
                        self.scope(&evidence.environment, expected, &value.name, item);
                    }
                }
            }
        }
    }

    pub(super) fn objections(&mut self) {
        let p = self.program;
        let scopes = objection_scopes(p);
        for value in p.objections.values() {
            let exists = match value.target_kind.as_str() {
                "claim" => p.claims.contains_key(&value.target),
                "reasoning" => p.reasoning.contains_key(&value.target),
                "assumption" => p.assumptions.contains_key(&value.target),
                "argument" => p.arguments.contains_key(&value.target),
                "objection" => p.objections.contains_key(&value.target),
                _ => {
                    self.detail("invalid_objection_target","Objection target kind must be claim, reasoning, assumption, argument or objection",Some(&value.name),None,Some(value.target_kind.clone()));
                    continue;
                }
            };
            let target_exists =
                self.reference(&value.target, exists, &value.target_kind, &value.name);
            if value.evidence.is_empty() && value.premises.is_empty() {
                self.error(
                    "empty_objection",
                    "An objection requires evidence or premise claims",
                    Some(&value.name),
                );
            }
            if scopes[&value.name].len() > 1 {
                self.error(
                    "environment_mismatch",
                    "An objection's evidence and premise claims must share one environment",
                    Some(&value.name),
                );
            }
            let mut expected = BTreeSet::new();
            if target_exists {
                match value.target_kind.as_str() {
                    "claim" => {
                        expected.insert(p.claims[&value.target].environment.clone());
                    }
                    "assumption" => {
                        expected.insert(p.assumptions[&value.target].environment.clone());
                    }
                    "argument" => {
                        if let Some(claim) = p.claims.get(&p.arguments[&value.target].conclusion) {
                            expected.insert(claim.environment.clone());
                        }
                    }
                    "objection" => expected.extend(scopes[&value.target].clone()),
                    _ => {}
                }
            }
            for item in &value.evidence {
                if self.reference(
                    item,
                    p.evidence.contains_key(item),
                    "objection evidence",
                    &value.name,
                ) {
                    for environment in &expected {
                        self.scope(
                            &p.evidence[item].environment,
                            environment,
                            &value.name,
                            item,
                        );
                    }
                }
            }
            for item in &value.premises {
                if self.reference(
                    item,
                    p.claims.contains_key(item),
                    "objection premise claim",
                    &value.name,
                ) {
                    for environment in &expected {
                        self.scope(&p.claims[item].environment, environment, &value.name, item);
                    }
                }
            }
        }
    }
}
