//! Predicate paths, method outputs and proposition metadata checks.
use super::{Checker, path, repr};
use crate::expressions::expression_errors_with_limits;
use serde_json::Value;
use std::collections::BTreeSet;

impl Checker<'_> {
    pub(super) fn predicates(&mut self) {
        let p = self.program;
        let entries = p
            .environments
            .values()
            .map(|v| (&v.name, &v.predicates, true))
            .chain(p.evidence.values().map(|v| (&v.name, &v.predicates, true)))
            .chain(
                p.reasoning
                    .values()
                    .map(|v| (&v.name, &v.predicates, false)),
            );
        for (owner, predicates, required) in entries {
            if required && predicates.is_empty() {
                self.error(
                    "missing_predicate",
                    "An environment or evidence declaration requires a predicate",
                    Some(owner),
                );
            }
            for predicate in predicates {
                if let Some(expression) = &predicate.expression {
                    for message in expression_errors_with_limits(expression, None, &p.limits) {
                        self.error("invalid_expression", message, Some(owner));
                    }
                    continue;
                }
                if !path(&predicate.path, true) {
                    self.error(
                        "invalid_path",
                        "Predicate paths must be dotted JSON object field names",
                        Some(owner),
                    );
                }
                if !["==", "!=", "<", "<=", ">", ">="].contains(&predicate.operator.as_str()) {
                    self.error(
                        "invalid_comparison",
                        "Predicate operator must be ==, !=, <, <=, > or >=",
                        Some(owner),
                    );
                }
                if predicate.expected.is_array() || predicate.expected.is_object() {
                    self.error(
                        "invalid_predicate",
                        "Predicate operands must be JSON scalar values",
                        Some(owner),
                    );
                }
                if let Value::Number(n) = &predicate.expected
                    && n.to_string().contains(['.', 'e', 'E'])
                    && !n.as_f64().is_some_and(f64::is_finite)
                {
                    self.error(
                        "invalid_number",
                        "Predicate numbers must be finite",
                        Some(owner),
                    );
                }
                if ["<", "<=", ">", ">="].contains(&predicate.operator.as_str())
                    && (predicate.expected.is_null() || predicate.expected.is_boolean())
                {
                    self.error(
                        "invalid_comparison",
                        "Ordered comparisons require a number or string",
                        Some(owner),
                    );
                }
            }
        }
    }

    pub(super) fn reasoning_and_claims(&mut self) {
        let p = self.program;
        for value in p.reasoning.values() {
            let contract = self.registry.get(&value.method);
            if !crate::methods::is_method_identifier(&value.method) {
                self.detail(
                    "invalid_method_reference",
                    "A method requires an explicit versioned identifier such as 'structured/1'",
                    Some(&value.name),
                    Some("versioned method identifier".into()),
                    Some(value.method.clone()),
                );
            } else if contract.is_none() {
                self.detail("unknown_method",format!("Unknown registered reasoning method {}; use an installed versioned identifier",repr(&value.method)),Some(&value.name),Some("registered versioned method identifier".into()),Some(value.method.clone()));
            }
            if let Some(contract) = contract {
                for predicate in &value.predicates {
                    if let Some(expression) = &predicate.expression {
                        for message in expression_errors_with_limits(
                            expression,
                            Some(&contract.output_schema),
                            &p.limits,
                        ) {
                            self.error("reasoning_predicate_type", message, Some(&value.name));
                        }
                        continue;
                    }
                    let parts: Vec<_> = predicate.path.split('.').collect();
                    if let Some(types) = output_scalar_types(&contract.output_schema, &parts) {
                        if types.is_empty() {
                            self.detail(
                                "reasoning_predicate_path",
                                format!(
                                    "Method {} has no scalar output at {}",
                                    repr(&value.method),
                                    repr(&predicate.path)
                                ),
                                Some(&value.name),
                                Some("declared scalar method output".into()),
                                Some(predicate.path.clone()),
                            );
                            continue;
                        }
                        let actual = scalar_type(&predicate.expected);
                        let allowed: BTreeSet<_> = types
                            .into_iter()
                            .map(|kind| {
                                if kind == "integer" {
                                    "number".into()
                                } else {
                                    kind
                                }
                            })
                            .collect();
                        if !allowed.contains(actual) {
                            self.detail(
                                "reasoning_predicate_type",
                                format!(
                                    "Predicate for {} has an incompatible operand type",
                                    repr(&predicate.path)
                                ),
                                Some(&value.name),
                                Some(allowed.into_iter().collect::<Vec<_>>().join(" | ")),
                                Some(actual.into()),
                            );
                        }
                    }
                }
            }
            if value.rationale.trim().is_empty() {
                self.error(
                    "empty_rationale",
                    "A reasoning declaration requires its rationale",
                    Some(&value.name),
                );
            }
            for item in &value.backing {
                self.reference(
                    item,
                    p.evidence.contains_key(item),
                    "backing evidence",
                    &value.name,
                );
            }
            if let Some(relation) = &value.transfer {
                self.reference(
                    &relation.source,
                    p.environments.contains_key(&relation.source),
                    "source environment",
                    &value.name,
                );
                self.reference(
                    &relation.target,
                    p.environments.contains_key(&relation.target),
                    "target environment",
                    &value.name,
                );
                self.reference(
                    &relation.assumption,
                    p.assumptions.contains_key(&relation.assumption),
                    "transfer assumption",
                    &value.name,
                );
                if relation.review.trim().is_empty() {
                    self.error(
                        "missing_review",
                        "A scope-transfer relation requires a review reference",
                        Some(&value.name),
                    );
                }
            }
        }
        for value in p.claims.values() {
            if value.statement.trim().is_empty() {
                self.error(
                    "empty_statement",
                    "A claim requires a statement",
                    Some(&value.name),
                );
            }
            self.reference(
                &value.environment,
                p.environments.contains_key(&value.environment),
                "environment",
                &value.name,
            );
            if let Some(proposition) = &value.proposition {
                if let Some(expression) = &proposition.result.expression {
                    for message in expression_errors_with_limits(expression, None, &p.limits) {
                        self.error("invalid_expression", message, Some(&value.name));
                    }
                } else {
                    if !path(&proposition.result.path, true) {
                        self.error(
                            "invalid_path",
                            "Proposition result paths must be dotted JSON object field names",
                            Some(&value.name),
                        );
                    }
                    if !["==", "!=", "<", "<=", ">", ">="]
                        .contains(&proposition.result.operator.as_str())
                    {
                        self.error(
                            "invalid_comparison",
                            "Proposition operator must be ==, !=, <, <=, > or >=",
                            Some(&value.name),
                        );
                    }
                }
                for message in crate::propositions::proposition_errors_with_limits(
                    proposition,
                    None,
                    self.registry,
                    &p.limits,
                ) {
                    self.error("invalid_proposition", message, Some(&value.name));
                }
            }
        }
    }
}

fn scalar_type(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "list",
        Value::Object(_) => "dict",
    }
}

fn output_scalar_types(schema: &Value, path: &[&str]) -> Option<BTreeSet<String>> {
    if let Some(choices) = schema.get("anyOf").and_then(Value::as_array) {
        let mut all = BTreeSet::new();
        for choice in choices {
            all.extend(output_scalar_types(choice, path)?);
        }
        return Some(all);
    }
    let kind = schema.get("type").and_then(Value::as_str).unwrap_or("");
    if kind == "json" {
        return None;
    }
    if path.is_empty() {
        return Some(
            if ["number", "integer", "boolean", "string", "null"].contains(&kind) {
                BTreeSet::from([kind.into()])
            } else {
                BTreeSet::new()
            },
        );
    }
    if kind != "object" {
        return Some(BTreeSet::new());
    }
    schema
        .get("properties")
        .and_then(|props| props.get(path[0]))
        .or_else(|| schema.get("additionalProperties"))
        .filter(|child| child.is_object())
        .map_or_else(
            || Some(BTreeSet::new()),
            |child| output_scalar_types(child, &path[1..]),
        )
}
