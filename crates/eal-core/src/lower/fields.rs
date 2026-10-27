//! Check authored fields and required fields inherited from lexical contexts.
use super::*;

impl Builder<'_> {
    pub(super) fn fields(&mut self, id: NodeId, wrapper: &str, kind: &str) -> Result<Vec<Field>> {
        let mut fields = Vec::new();
        let mut seen = BTreeSet::new();
        for wrapper_id in self.children(id, wrapper) {
            let child = self
                .syntax
                .children(wrapper_id)
                .next()
                .ok_or_else(|| self.error(wrapper_id, "SYNTAX_TREE", "Empty field"))?;
            let field_id = if self.syntax.node(child).rule_name().is_some() {
                child
            } else {
                wrapper_id
            };
            let field = self.field(field_id)?;
            if !matches!(field.name(), "require" | "bind") && !seen.insert(field.name()) {
                return Err(self.error(
                    field_id,
                    "DUPLICATE_FIELD",
                    format!("Duplicate field {:?}", field.name()),
                ));
            }
            fields.push(field);
        }
        let inherited: &[&str] = match kind {
            "tool" => &["version"],
            "evidence" => &["environment", "tool", "max_age", "kind", "input"],
            "assumption" => &["environment", "validate", "valid_from", "valid_until"],
            "claim" => &["environment"],
            "reasoning" => &["method"],
            "proposition" => &[
                "subject",
                "quantity",
                "unit",
                "scope",
                "valid_from",
                "valid_until",
                "query",
            ],
            _ => &[],
        };
        let required: &[&str] = match kind {
            "tool" => &["version"],
            "evidence" => &["kind", "environment", "max_age"],
            "assumption" => &["statement", "environment", "validate"],
            "reasoning" => &["method", "rationale"],
            "claim" => &["statement", "environment"],
            "proposition" => &[
                "subject",
                "quantity",
                "unit",
                "scope",
                "valid_from",
                "valid_until",
                "result",
                "query",
            ],
            _ => &[],
        };
        let missing: Vec<_> = required
            .iter()
            .filter(|&&name| {
                !(seen.contains(name)
                    || inherited.contains(&name) && self.defaults.contains_key(name))
            })
            .copied()
            .collect();
        if !missing.is_empty() {
            return Err(self.error(
                id,
                "MISSING_FIELD",
                format!(
                    "Missing field after context inheritance: {}",
                    missing.join(", ")
                ),
            ));
        }
        if kind == "evidence" {
            let derive = seen.contains("derive");
            let tool = seen.contains("tool") || !derive && self.defaults.contains_key("tool");
            if tool == derive {
                return Err(self.error(
                    id,
                    "EVIDENCE_SOURCE",
                    "Evidence requires exactly one tool or derive field",
                ));
            }
        }
        if kind == "evidence"
            && !fields
                .iter()
                .any(|field| matches!(field, Field::Require { .. }))
        {
            return Err(self.error(
                id,
                "MISSING_PREDICATE",
                "Evidence requires at least one require predicate",
            ));
        }
        Ok(fields)
    }

    pub(super) fn field(&mut self, id: NodeId) -> Result<Field> {
        use TextProperty as T;
        let name = self.first_text(id);
        let text_property = match name.as_str() {
            "version" => Some(T::Version),
            "statement" => Some(T::Statement),
            "valid_from" => Some(T::ValidFrom),
            "valid_until" => Some(T::ValidUntil),
            "method" => Some(T::Method),
            "rationale" => Some(T::Rationale),
            "subject" => Some(T::Subject),
            "quantity" => Some(T::Quantity),
            "unit" => Some(T::Unit),
            "scope" => Some(T::Scope),
            _ => None,
        };
        if let Some(property) = text_property {
            return Ok(Field::Text {
                property,
                value: self.string(id)?,
            });
        }
        Ok(match name.as_str() {
            "tool" => Field::Name {
                property: NameProperty::Tool,
                value: self.name(id)?,
            },
            "environment" => Field::Name {
                property: NameProperty::Environment,
                value: self.name(id)?,
            },
            "validate" => Field::Name {
                property: NameProperty::Validate,
                value: self.name(id)?,
            },
            "kind" => Field::Kind {
                value: self.text(self.child(id, "identifier")?),
            },
            "max_age" => Field::MaxAge {
                value: NumberLiteral(self.text(self.child(id, "signedNumber")?)),
            },
            "input" => Field::Json {
                property: JsonProperty::Input,
                value: self.json(self.child(id, "jsonValue")?)?,
            },
            "query" => Field::Json {
                property: JsonProperty::Query,
                value: self.json(self.child(id, "jsonValue")?)?,
            },
            "derive" => Field::Derive {
                expression: self.expression(self.child(id, "expression")?)?,
            },
            "bind" => {
                let strings = self.string_tokens(id);
                Field::InputBinding {
                    target: self.decode_string(strings[0])?,
                    evidence: self.name(id)?,
                    source: self.decode_string(strings[1])?,
                }
            }
            "require" => Field::Require {
                expression: self.expression(self.child(id, "expression")?)?,
            },
            "result" => Field::Result {
                expression: self.expression(self.child(id, "expression")?)?,
            },
            "backing" => Field::Backing {
                references: self.references(self.child(id, "referenceList")?)?,
            },
            "transfer" => {
                let names = self.children(id, "qualifiedName");
                Field::Transfer {
                    source: self.text(names[0]),
                    target: self.text(names[1]),
                    assumption: self.text(names[2]),
                    review: self.string(id)?,
                }
            }
            "proposition" => Field::Proposition {
                fields: self.fields(id, "propositionField", "proposition")?,
            },
            _ => return Err(self.error(id, "SYNTAX_TREE", format!("Unknown field {name}"))),
        })
    }
}
