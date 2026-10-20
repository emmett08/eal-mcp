//! Resolve typed scalar and sequence references through lexical scope.
use super::*;
impl Lowerer<'_> {
    pub(super) fn resolve(
        &mut self,
        reference: &str,
        record: &Record,
        many: bool,
    ) -> s::BindingValue {
        let value = if let Some((op, name)) = operation(reference) {
            match record.scope.bindings.get(name) {
                Some(s::BindingValue::List(values)) => {
                    if op == "tail" {
                        s::BindingValue::List(values.iter().skip(1).cloned().collect())
                    } else if let Some(head) = values.first() {
                        s::BindingValue::Single(head.clone())
                    } else {
                        self.error(
                            "empty_pattern_head",
                            "head requires a nonempty guarded list",
                            record.scope.application.as_deref(),
                            Some(record.scope.span),
                            None,
                            None,
                        );
                        return if many {
                            s::BindingValue::List(Vec::new())
                        } else {
                            s::BindingValue::Single(reference.into())
                        };
                    }
                }
                _ => {
                    self.error(
                        "pattern_reference_kind",
                        format!("{op} requires a list parameter"),
                        record
                            .scope
                            .application
                            .as_deref()
                            .or(Some(record.value.name())),
                        Some(record.scope.span),
                        None,
                        None,
                    );
                    return if many {
                        s::BindingValue::List(Vec::new())
                    } else {
                        s::BindingValue::Single(reference.into())
                    };
                }
            }
        } else if let Some(bound) = record.scope.bindings.get(reference) {
            bound.clone()
        } else {
            s::BindingValue::Single(
                candidates(&record.scope.prefix, reference)
                    .into_iter()
                    .find(|n| self.symbols.contains_key(n))
                    .unwrap_or_else(|| reference.into()),
            )
        };
        match (value, many) {
            (s::BindingValue::Single(v), true) => s::BindingValue::List(vec![v]),
            (s::BindingValue::List(_), false) => {
                self.error(
                    "pattern_reference_kind",
                    "A scalar reference cannot consume a list",
                    record
                        .scope
                        .application
                        .as_deref()
                        .or(Some(record.value.name())),
                    Some(record.scope.span),
                    None,
                    None,
                );
                s::BindingValue::Single(reference.into())
            }
            (v, _) => v,
        }
    }
    pub(super) fn scalar(&mut self, reference: &str, record: &Record) -> String {
        match self.resolve(reference, record, false) {
            s::BindingValue::Single(v) => v,
            _ => unreachable!(),
        }
    }
    pub(super) fn sequence(&mut self, references: &[String], record: &Record) -> Vec<String> {
        references
            .iter()
            .flat_map(|r| match self.resolve(r, record, true) {
                s::BindingValue::List(v) => v,
                _ => unreachable!(),
            })
            .collect()
    }
    pub(super) fn resolved(&mut self, record: &Record) -> Entity {
        let mut value = record.value.clone();
        match &mut value {
            Entity::Evidence(v) => {
                v.tool = self.scalar(&v.tool, record);
                v.environment = self.scalar(&v.environment, record);
            }
            Entity::Claim(v) => v.environment = self.scalar(&v.environment, record),
            Entity::Assumption(v) => {
                v.environment = self.scalar(&v.environment, record);
                v.validation = self.scalar(&v.validation, record);
            }
            Entity::Reasoning(v) => {
                v.backing = self.sequence(&v.backing, record);
                if let Some(t) = &mut v.transfer {
                    t.source = self.scalar(&t.source, record);
                    t.target = self.scalar(&t.target, record);
                    t.assumption = self.scalar(&t.assumption, record);
                }
            }
            Entity::Argument(v) => {
                v.conclusion = self.scalar(&v.conclusion, record);
                v.reasoning = self.scalar(&v.reasoning, record);
                v.evidence = self.sequence(&v.evidence, record);
                v.assumptions = self.sequence(&v.assumptions, record);
                v.premises = self.sequence(&v.premises, record);
                v.binding = v.binding.as_ref().map(|b| self.scalar(b, record));
            }
            Entity::Objection(v) => {
                v.target = self.scalar(&v.target, record);
                v.evidence = self.sequence(&v.evidence, record);
                v.premises = self.sequence(&v.premises, record);
            }
            Entity::Application(v) => {
                for binding in &mut v.arguments {
                    binding.reference = match &binding.reference {
                        s::BindingValue::List(values) => {
                            s::BindingValue::List(self.sequence(values, record))
                        }
                        s::BindingValue::Single(name) => self.resolve(
                            name,
                            record,
                            operation(name).is_some_and(|(op, _)| op == "tail"),
                        ),
                    };
                }
                v.pattern = candidates(&record.scope.prefix, &v.pattern)
                    .into_iter()
                    .find(|n| self.records["pattern"].contains_key(n))
                    .unwrap_or_else(|| v.pattern.clone());
            }
            Entity::Directive(v) => {
                v.name = self.scalar(&v.name, record);
                v.other = v.other.as_ref().map(|n| self.scalar(n, record));
            }
            _ => {}
        }
        value
    }
}
