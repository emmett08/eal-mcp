//! Translate typed lexical fields into their composed semantic entities.
use super::*;
use crate::{
    ArgumentFlow, BindingValue, Directive, JsonValue, PatternBody, Reference, SupportKind,
};
use serde_json::Value;

pub(super) fn reference(reference: &Reference) -> String {
    match reference {
        Reference::Name(n) => n.clone(),
        Reference::Head(n) => format!("head({n})"),
        Reference::Tail(n) => format!("tail({n})"),
    }
}
fn literal(value: &JsonValue) -> Result<Value> {
    Ok(match value {
        JsonValue::Null => Value::Null,
        JsonValue::Boolean(v) => Value::Bool(*v),
        JsonValue::String(v) => Value::String(v.clone()),
        JsonValue::Number(v) => serde_json::from_str(&v.0).map_err(|error| {
            structure("invalid_number", error.to_string(), SourceSpan::default())
        })?,
        JsonValue::Array(v) => Value::Array(v.iter().map(literal).collect::<Result<_>>()?),
        JsonValue::Object(v) => Value::Object(
            v.iter()
                .map(|(key, value)| Ok((key.clone(), literal(value)?)))
                .collect::<Result<_>>()?,
        ),
    })
}
fn inherit(kind: &str, name: &str) -> bool {
    match kind {
        "tool" => name == "version",
        "evidence" => matches!(name, "environment" | "tool" | "max_age" | "kind" | "input"),
        "assumption" => matches!(
            name,
            "environment" | "validate" | "valid_from" | "valid_until"
        ),
        "claim" => name == "environment",
        "reasoning" => name == "method",
        "proposition" => matches!(
            name,
            "subject" | "quantity" | "unit" | "scope" | "valid_from" | "valid_until" | "query"
        ),
        _ => false,
    }
}
struct Fields<'a> {
    fields: &'a [Field],
    defaults: &'a BTreeMap<String, Field>,
    kind: &'a str,
    span: SourceSpan,
}
impl<'a> Fields<'a> {
    fn get(&self, name: &str) -> Option<&Field> {
        self.fields.iter().find(|f| f.name() == name).or_else(|| {
            inherit(self.kind, name)
                .then(|| self.defaults.get(name))
                .flatten()
        })
    }
    fn text(&self, name: &str) -> Result<String> {
        match self.get(name) {
            Some(Field::Text { value, .. })
            | Some(Field::Name { value, .. })
            | Some(Field::Kind { value }) => Ok(value.clone()),
            _ => Err(structure(
                "MISSING_FIELD",
                format!("Missing field {name_repr}", name_repr = repr(name)),
                self.span,
            )),
        }
    }
    fn optional(&self, name: &str) -> Option<String> {
        self.get(name).and_then(|field| match field {
            Field::Text { value, .. } | Field::Name { value, .. } => Some(value.clone()),
            _ => None,
        })
    }
    fn predicates(&self) -> Vec<s::Predicate> {
        self.fields
            .iter()
            .filter_map(|field| match field {
                Field::Require { expression } => {
                    Some(crate::expressions::make_predicate(expression))
                }
                _ => None,
            })
            .collect()
    }
    fn proposition(&self) -> Result<Option<s::Proposition>> {
        let Some(Field::Proposition { fields }) = self.get("proposition") else {
            return Ok(None);
        };
        let nested = Fields {
            fields,
            defaults: self.defaults,
            kind: "proposition",
            span: self.span,
        };
        let result = match nested.get("result") {
            Some(Field::Result { expression }) => crate::expressions::make_predicate(expression),
            _ => {
                return Err(structure(
                    "MISSING_FIELD",
                    "Missing proposition result",
                    self.span,
                ));
            }
        };
        let query = match nested.get("query") {
            Some(Field::Json { value, .. }) => literal(value)?,
            _ => {
                return Err(structure(
                    "MISSING_FIELD",
                    "Missing proposition query",
                    self.span,
                ));
            }
        };
        Ok(Some(s::Proposition {
            subject: nested.text("subject")?,
            quantity: nested.text("quantity")?,
            unit: nested.text("unit")?,
            scope: nested.text("scope")?,
            valid_from: nested.text("valid_from")?,
            valid_until: nested.text("valid_until")?,
            result,
            query,
        }))
    }
}
pub(super) fn argument(name: &str, flow: &ArgumentFlow) -> s::Argument {
    let group = |kind| {
        flow.support
            .iter()
            .find(|s| s.kind == kind)
            .map_or_else(Vec::new, |s| s.references.iter().map(reference).collect())
    };
    s::Argument {
        name: name.into(),
        conclusion: reference(&flow.conclusion),
        reasoning: reference(&flow.reasoning),
        evidence: group(SupportKind::Evidence),
        assumptions: group(SupportKind::Assumptions),
        premises: group(SupportKind::Premises),
        binding: flow.binding.as_ref().map(reference),
        origin: None,
    }
}
pub(super) fn pattern(
    name: &str,
    parameters: &[crate::PatternParameter],
    decreases: &Option<String>,
    body: &PatternBody,
    defaults: &BTreeMap<String, Field>,
) -> s::Pattern {
    let flow = if let PatternBody::Flow(flow) = body {
        Some(argument(name, flow))
    } else {
        None
    };
    s::Pattern {
        name: name.into(),
        parameters: parameters
            .iter()
            .map(|p| s::PatternParameter {
                name: p.name.clone(),
                kind: format!("{}{}", p.kind, if p.list { "[]" } else { "" }),
            })
            .collect(),
        conclusion: flow
            .as_ref()
            .map_or_else(String::new, |f| f.conclusion.clone()),
        reasoning: flow
            .as_ref()
            .map_or_else(String::new, |f| f.reasoning.clone()),
        evidence: flow.as_ref().map_or_else(Vec::new, |f| f.evidence.clone()),
        assumptions: flow
            .as_ref()
            .map_or_else(Vec::new, |f| f.assumptions.clone()),
        premises: flow.as_ref().map_or_else(Vec::new, |f| f.premises.clone()),
        binding: flow.and_then(|f| f.binding),
        body: if let PatternBody::Block(body) = body {
            Some(body.clone())
        } else {
            None
        },
        decreases: decreases.clone(),
        defaults: defaults.clone(),
    }
}
pub(super) fn entity(
    declaration: &Declaration,
    defaults: &BTreeMap<String, Field>,
) -> Result<Entity> {
    let span = declaration.span;
    let fields = |fields, kind| Fields {
        fields,
        defaults,
        kind,
        span,
    };
    Ok(match &declaration.value {
        DeclarationKind::Environment { name, predicates } => Entity::Environment(s::Environment {
            name: name.clone(),
            predicates: predicates
                .iter()
                .map(crate::expressions::make_predicate)
                .collect(),
        }),
        DeclarationKind::Tool {
            name,
            fields: values,
        } => {
            let f = fields(values.as_slice(), "tool");
            Entity::Tool(s::Tool {
                name: name.clone(),
                version: f.text("version")?,
            })
        }
        DeclarationKind::Evidence {
            name,
            fields: values,
        } => {
            let f = fields(values.as_slice(), "evidence");
            let max_age = match f.get("max_age") {
                Some(Field::MaxAge { value }) => value
                    .0
                    .parse::<f64>()
                    .map_err(|error| structure("invalid_number", error.to_string(), span))?,
                _ => return Err(structure("MISSING_FIELD", "Missing max_age", span)),
            };
            let input = match f.get("input") {
                Some(Field::Json { value, .. }) => literal(value)?,
                _ => serde_json::json!({}),
            };
            Entity::Evidence(s::Evidence {
                name: name.clone(),
                tool: if values
                    .iter()
                    .any(|field| matches!(field, Field::Derive { .. }))
                {
                    None
                } else {
                    f.optional("tool")
                },
                derivation: values.iter().find_map(|field| match field {
                    Field::Derive { expression } => Some(expression.clone()),
                    _ => None,
                }),
                kind: f.text("kind")?,
                environment: f.text("environment")?,
                max_age,
                input,
                bindings: values
                    .iter()
                    .filter_map(|field| match field {
                        Field::InputBinding {
                            target,
                            evidence,
                            source,
                        } => Some(s::InputBinding {
                            target: target.clone(),
                            evidence: evidence.clone(),
                            source: source.clone(),
                        }),
                        _ => None,
                    })
                    .collect(),
                predicates: f.predicates(),
            })
        }
        DeclarationKind::Assumption {
            name,
            fields: values,
        } => {
            let f = fields(values.as_slice(), "assumption");
            Entity::Assumption(s::Assumption {
                name: name.clone(),
                statement: f.text("statement")?,
                environment: f.text("environment")?,
                validation: f.text("validate")?,
                valid_from: f.optional("valid_from"),
                valid_until: f.optional("valid_until"),
            })
        }
        DeclarationKind::Reasoning {
            name,
            fields: values,
        } => {
            let f = fields(values.as_slice(), "reasoning");
            let backing = match f.get("backing") {
                Some(Field::Backing { references }) => references.iter().map(reference).collect(),
                _ => Vec::new(),
            };
            let transfer = match f.get("transfer") {
                Some(Field::Transfer {
                    source,
                    target,
                    assumption,
                    review,
                }) => Some(s::ScopeTransfer {
                    source: source.clone(),
                    target: target.clone(),
                    assumption: assumption.clone(),
                    review: review.clone(),
                }),
                _ => None,
            };
            Entity::Reasoning(s::Reasoning {
                name: name.clone(),
                method: f.text("method")?,
                rationale: f.text("rationale")?,
                backing,
                predicates: f.predicates(),
                transfer,
            })
        }
        DeclarationKind::Claim {
            name,
            fields: values,
        } => {
            let f = fields(values.as_slice(), "claim");
            Entity::Claim(s::Claim {
                name: name.clone(),
                statement: f.text("statement")?,
                environment: f.text("environment")?,
                proposition: f.proposition()?,
            })
        }
        DeclarationKind::Argument { name, flow, .. } => Entity::Argument(argument(name, flow)),
        DeclarationKind::Pattern {
            name,
            parameters,
            decreases,
            body,
        } => Entity::Pattern(pattern(name, parameters, decreases, body, defaults)),
        DeclarationKind::Application {
            name,
            pattern,
            bindings,
        } => Entity::Application(s::Application {
            name: name.clone(),
            pattern: pattern.clone(),
            arguments: bindings
                .iter()
                .map(|binding| s::PatternBinding {
                    name: binding.name.clone(),
                    reference: match &binding.value {
                        BindingValue::Single(v) => s::BindingValue::Single(reference(v)),
                        BindingValue::List(v) => {
                            s::BindingValue::List(v.iter().map(reference).collect())
                        }
                    },
                })
                .collect(),
        }),
        DeclarationKind::Objection {
            name,
            support,
            target_kind,
            target,
        } => {
            let group = |kind| {
                support
                    .iter()
                    .find(|s| s.kind == kind)
                    .map_or_else(Vec::new, |s| s.references.iter().map(reference).collect())
            };
            Entity::Objection(s::Objection {
                name: name.clone(),
                target_kind: match target_kind {
                    crate::TargetKind::Claim => "claim",
                    crate::TargetKind::Reasoning => "reasoning",
                    crate::TargetKind::Assumption => "assumption",
                    crate::TargetKind::Argument => "argument",
                    crate::TargetKind::Objection => "objection",
                }
                .into(),
                target: reference(target),
                evidence: group(SupportKind::Evidence),
                premises: group(SupportKind::Premises),
            })
        }
        DeclarationKind::Argumentation { directive } => {
            let (kind, name, other, rank, review) = match directive {
                Directive::Strict { name, review } => ("strict", name, None, None, review),
                Directive::Rank {
                    name,
                    value,
                    review,
                } => (
                    "rank",
                    name,
                    None,
                    value
                        .0
                        .chars()
                        .all(|c| c.is_ascii_digit())
                        .then(|| value.0.parse::<i64>().ok())
                        .flatten(),
                    review,
                ),
                Directive::Contrary {
                    name,
                    other,
                    review,
                } => ("contrary", name, Some(other.clone()), None, review),
                Directive::Prefer {
                    name,
                    other,
                    review,
                } => ("prefer", name, Some(other.clone()), None, review),
            };
            Entity::Directive(s::ArgumentationDirective {
                kind: kind.into(),
                name: name.clone(),
                other,
                rank,
                review: review.clone(),
                span,
                source_file: None,
            })
        }
        _ => {
            return Err(structure(
                "DECLARATION",
                "Expected atomic declaration",
                span,
            ));
        }
    })
}
pub(super) fn insert(program: &mut s::Program, value: Entity) {
    match value {
        Entity::Environment(v) => {
            program.environments.insert(v.name.clone(), v);
        }
        Entity::Tool(v) => {
            program.tools.insert(v.name.clone(), v);
        }
        Entity::Evidence(v) => {
            program.evidence.insert(v.name.clone(), v);
        }
        Entity::Assumption(v) => {
            program.assumptions.insert(v.name.clone(), v);
        }
        Entity::Reasoning(v) => {
            program.reasoning.insert(v.name.clone(), v);
        }
        Entity::Claim(v) => {
            program.claims.insert(v.name.clone(), v);
        }
        Entity::Argument(v) => {
            program.arguments.insert(v.name.clone(), v);
        }
        Entity::Objection(v) => {
            program.objections.insert(v.name.clone(), v);
        }
        Entity::Pattern(v) => {
            program.patterns.insert(v.name.clone(), v);
        }
        Entity::Application(v) => {
            program.applications.insert(v.name.clone(), v);
        }
        Entity::Directive(v) => program.argumentation_directives.push(v),
    }
}

/// Local metadata uses the same conversion as actual expansion and performs no bindings.
pub fn pattern_metadata(
    pattern: &s::Pattern,
    limits: &ExecutionLimits,
) -> Result<Vec<(String, s::Program, SourceSpan)>> {
    fn walk(
        declarations: &[Declaration],
        defaults: &BTreeMap<String, Field>,
        limits: &ExecutionLimits,
        out: &mut Vec<(String, s::Program, SourceSpan)>,
    ) -> Result<()> {
        for declaration in declarations.iter().rev() {
            match &declaration.value {
                DeclarationKind::Context {
                    defaults: local,
                    body,
                } => {
                    let mut inherited = defaults.clone();
                    for f in local {
                        inherited.insert(f.name().into(), f.clone());
                    }
                    walk(body, &inherited, limits, out)?;
                }
                DeclarationKind::Module { body, .. }
                | DeclarationKind::Argument {
                    body: Some(body), ..
                }
                | DeclarationKind::Pattern {
                    body: PatternBody::Block(body),
                    ..
                } => walk(body, defaults, limits, out)?,
                DeclarationKind::Guard {
                    body, otherwise, ..
                } => {
                    if let Some(otherwise) = otherwise {
                        walk(otherwise, defaults, limits, out)?;
                    }
                    walk(body, defaults, limits, out)?;
                }
                DeclarationKind::Environment { .. }
                | DeclarationKind::Tool { .. }
                | DeclarationKind::Evidence { .. }
                | DeclarationKind::Assumption { .. }
                | DeclarationKind::Reasoning { .. }
                | DeclarationKind::Claim { .. } => {
                    let value = entity(declaration, defaults)?;
                    let name = value.name().to_owned();
                    let mut program =
                        s::Program::empty(crate::LANGUAGE.into(), String::new(), limits.clone());
                    insert(&mut program, value);
                    out.push((name, program, declaration.span));
                }
                _ => {}
            }
        }
        Ok(())
    }
    let mut result = Vec::new();
    if let Some(body) = &pattern.body {
        walk(body, &pattern.defaults, limits, &mut result)?;
    }
    Ok(result)
}
