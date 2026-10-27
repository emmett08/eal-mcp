//! Convert recognised grammar rules into typed authored EAL syntax.
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use eal_parser::{NodeId, ParseError, SourceSpan, SyntaxDocument};

use crate::ast::*;

mod fields;
mod values;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CoreError {
    Recognition(ParseError),
    Structure {
        code: &'static str,
        message: String,
        span: SourceSpan,
    },
}

impl fmt::Display for CoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Recognition(error) => write!(f, "{error}"),
            Self::Structure {
                code,
                message,
                span,
            } => write!(
                f,
                "{}:{}: {code}: {message}",
                span.start.line, span.start.column
            ),
        }
    }
}

impl std::error::Error for CoreError {}

type Result<T> = std::result::Result<T, CoreError>;

/// Bound the recursive typed representation as well as the concrete parser stack.
/// Flat operator chains can otherwise create a deep tree from shallow syntax.
pub const MAX_EXPRESSION_DEPTH: usize = 128;

/// Lower strictly recognised syntax. Imports remain authored references to host-owned sources.
pub fn lower(syntax: &SyntaxDocument) -> Result<Program> {
    std::thread::scope(|scope| {
        let worker = std::thread::Builder::new()
            .name("eal-lowering".to_owned())
            .stack_size(16 * 1024 * 1024)
            .spawn_scoped(scope, || lower_owned(syntax))
            .map_err(|error| CoreError::Structure {
                code: "LOWERING_RESOURCE",
                message: format!("Cannot start bounded lowering worker: {error}"),
                span: syntax.node(syntax.root).span,
            })?;
        worker.join().map_err(|_| CoreError::Structure {
            code: "LOWERING_RESOURCE",
            message: "Lowering worker failed".to_owned(),
            span: syntax.node(syntax.root).span,
        })?
    })
}

fn lower_owned(syntax: &SyntaxDocument) -> Result<Program> {
    let mut builder = Builder {
        syntax,
        defaults: BTreeMap::new(),
    };
    let root = syntax.root;
    let language = builder.string(root)?;
    if language != crate::LANGUAGE {
        return Err(builder.error(root, "LANGUAGE_VERSION", format!("Source must declare {}; migrate older source and imported modules to the current language", crate::LANGUAGE)));
    }
    Ok(Program {
        language,
        declarations: builder.block(root)?,
    })
}

struct Builder<'a> {
    syntax: &'a SyntaxDocument,
    defaults: BTreeMap<String, Field>,
}

impl Builder<'_> {
    fn error(&self, id: NodeId, code: &'static str, message: impl Into<String>) -> CoreError {
        CoreError::Structure {
            code,
            message: message.into(),
            span: self.syntax.node(id).span,
        }
    }

    fn children(&self, id: NodeId, name: &str) -> Vec<NodeId> {
        self.syntax
            .children(id)
            .filter(|&n| self.syntax.node(n).rule_name() == Some(name))
            .collect()
    }

    fn child(&self, id: NodeId, name: &str) -> Result<NodeId> {
        self.children(id, name)
            .first()
            .copied()
            .ok_or_else(|| self.error(id, "SYNTAX_TREE", format!("Missing {name} node")))
    }

    fn text(&self, id: NodeId) -> String {
        if let Some(token) = self.syntax.token(id) {
            return if token.kind == "EOF" {
                String::new()
            } else {
                token.text.clone()
            };
        }
        self.syntax
            .children(id)
            .map(|child| self.text(child))
            .collect()
    }

    fn first_text(&self, id: NodeId) -> String {
        if let Some(token) = self.syntax.token(id) {
            return token.text.clone();
        }
        self.syntax
            .children(id)
            .next()
            .map(|child| self.first_text(child))
            .unwrap_or_default()
    }

    fn has_token(&self, id: NodeId, value: &str) -> bool {
        self.syntax.children(id).any(|child| {
            self.syntax
                .token(child)
                .is_some_and(|token| token.text == value)
        })
    }

    fn string_tokens(&self, id: NodeId) -> Vec<NodeId> {
        self.syntax
            .children(id)
            .filter(|&child| {
                self.syntax
                    .token(child)
                    .is_some_and(|token| token.kind == "STRING")
            })
            .collect()
    }

    fn string_token(&self, id: NodeId) -> Result<NodeId> {
        self.syntax
            .children(id)
            .find(|&child| {
                self.syntax
                    .token(child)
                    .is_some_and(|token| token.kind == "STRING")
            })
            .ok_or_else(|| self.error(id, "SYNTAX_TREE", "Missing string token"))
    }

    fn decode_string(&self, id: NodeId) -> Result<String> {
        serde_json::from_str(&self.text(id))
            .map_err(|error| self.error(id, "STRING", format!("Invalid Unicode string: {error}")))
    }

    fn string(&self, id: NodeId) -> Result<String> {
        self.decode_string(self.string_token(id)?)
    }

    fn name(&self, id: NodeId) -> Result<String> {
        Ok(self.text(self.child(id, "qualifiedName")?))
    }

    fn block(&mut self, id: NodeId) -> Result<Vec<Declaration>> {
        self.children(id, "declaration")
            .into_iter()
            .map(|child| self.declaration(child))
            .collect()
    }

    fn declaration(&mut self, id: NodeId) -> Result<Declaration> {
        let value_id = self
            .syntax
            .children(id)
            .next()
            .ok_or_else(|| self.error(id, "SYNTAX_TREE", "Empty declaration"))?;
        let value = match self.syntax.node(value_id).rule_name().unwrap_or("") {
            "moduleDecl" => DeclarationKind::Module {
                name: self.name(value_id)?,
                body: self.block(value_id)?,
            },
            "importDecl" => {
                let path = self.string(value_id)?;
                if path.is_empty() || path.contains('\\') || path.starts_with('/') {
                    return Err(self.error(
                        value_id,
                        "IMPORT_PATH",
                        "Imports require a relative POSIX source path",
                    ));
                }
                DeclarationKind::Import {
                    path,
                    alias: self.text(self.child(value_id, "identifier")?),
                }
            }
            "contextDecl" => {
                let mut defaults = Vec::new();
                let mut seen = BTreeSet::new();
                for attribute in self.children(value_id, "contextAttribute") {
                    let field = self.field(attribute)?;
                    if !seen.insert(field.name()) {
                        return Err(self.error(
                            attribute,
                            "DUPLICATE_CONTEXT",
                            format!("Duplicate context default {:?}", field.name()),
                        ));
                    }
                    defaults.push(field);
                }
                let previous = self.defaults.clone();
                for field in &defaults {
                    self.defaults.insert(field.name().to_owned(), field.clone());
                }
                let body = self.block(value_id);
                self.defaults = previous;
                DeclarationKind::Context {
                    defaults,
                    body: body?,
                }
            }
            "environmentDecl" => {
                let predicates = self
                    .children(value_id, "predicate")
                    .into_iter()
                    .map(|p| self.expression(self.child(p, "expression")?))
                    .collect::<Result<_>>()?;
                DeclarationKind::Environment {
                    name: self.name(value_id)?,
                    predicates,
                }
            }
            "toolDecl" => DeclarationKind::Tool {
                name: self.name(value_id)?,
                fields: self.fields(value_id, "versionField", "tool")?,
            },
            "evidenceDecl" => DeclarationKind::Evidence {
                name: self.name(value_id)?,
                fields: self.fields(value_id, "evidenceField", "evidence")?,
            },
            "assumptionDecl" => DeclarationKind::Assumption {
                name: self.name(value_id)?,
                fields: self.fields(value_id, "assumptionField", "assumption")?,
            },
            "reasoningDecl" => DeclarationKind::Reasoning {
                name: self.name(value_id)?,
                fields: self.fields(value_id, "reasoningField", "reasoning")?,
            },
            "claimDecl" => DeclarationKind::Claim {
                name: self.name(value_id)?,
                fields: self.fields(value_id, "claimField", "claim")?,
            },
            "argumentDecl" => DeclarationKind::Argument {
                name: self.name(value_id)?,
                body: if self.has_token(value_id, "{") {
                    Some(self.block(value_id)?)
                } else {
                    None
                },
                flow: self.flow(self.child(value_id, "argumentFlow")?)?,
            },
            "patternDecl" => {
                let parameters = self
                    .children(value_id, "patternParameter")
                    .into_iter()
                    .map(|p| {
                        Ok(PatternParameter {
                            name: self.text(self.child(p, "identifier")?),
                            kind: self.text(self.child(p, "parameterKind")?),
                            list: self.has_token(p, "["),
                        })
                    })
                    .collect::<Result<_>>()?;
                let decreases = self
                    .children(value_id, "identifier")
                    .first()
                    .map(|&n| self.text(n));
                let flow = self.children(value_id, "argumentFlow");
                let body = if let Some(&flow_id) = flow.first() {
                    PatternBody::Flow(self.flow(flow_id)?)
                } else {
                    PatternBody::Block(self.block(value_id)?)
                };
                DeclarationKind::Pattern {
                    name: self.name(value_id)?,
                    parameters,
                    decreases,
                    body,
                }
            }
            "applicationDecl" => {
                let names = self.children(value_id, "qualifiedName");
                let bindings = self
                    .children(value_id, "patternBinding")
                    .into_iter()
                    .map(|b| {
                        let v = self.child(b, "bindingValue")?;
                        let references = self.references(v)?;
                        let value = if self.has_token(v, "[") {
                            BindingValue::List(references)
                        } else {
                            BindingValue::Single(references.into_iter().next().ok_or_else(
                                || self.error(v, "SYNTAX_TREE", "Missing binding reference"),
                            )?)
                        };
                        Ok(PatternBinding {
                            name: self.text(self.child(b, "identifier")?),
                            value,
                        })
                    })
                    .collect::<Result<_>>()?;
                DeclarationKind::Application {
                    name: self.text(names[0]),
                    pattern: self.text(names[1]),
                    bindings,
                }
            }
            "patternGuard" => {
                let mut body = Vec::new();
                let mut otherwise = None;
                for child in self.syntax.children(value_id) {
                    if self
                        .syntax
                        .token(child)
                        .is_some_and(|token| token.text == "else")
                    {
                        otherwise = Some(Vec::new());
                    }
                    if self.syntax.node(child).rule_name() == Some("declaration") {
                        let declaration = self.declaration(child)?;
                        if let Some(after) = otherwise.as_mut() {
                            after.push(declaration);
                        } else {
                            body.push(declaration);
                        }
                    }
                }
                DeclarationKind::Guard {
                    parameter: self.text(self.child(value_id, "identifier")?),
                    body,
                    otherwise,
                }
            }
            "objectionDecl" => {
                let kind = self.text(self.child(value_id, "targetKind")?);
                let target_kind = match kind.as_str() {
                    "claim" => TargetKind::Claim,
                    "reasoning" => TargetKind::Reasoning,
                    "assumption" => TargetKind::Assumption,
                    "argument" => TargetKind::Argument,
                    "objection" => TargetKind::Objection,
                    _ => {
                        return Err(self.error(
                            value_id,
                            "SYNTAX_TREE",
                            "Unknown objection target kind",
                        ));
                    }
                };
                DeclarationKind::Objection {
                    name: self.name(value_id)?,
                    support: self
                        .groups(self.child(value_id, "objectionSupport")?, "objectionGroup")?,
                    target_kind,
                    target: self.reference(self.child(value_id, "reference")?)?,
                }
            }
            "argumentationDirective" => {
                let names = self.children(value_id, "qualifiedName");
                let name = self.text(names[0]);
                let review = self.string(value_id)?;
                let directive = match self.first_text(value_id).as_str() {
                    "strict" => Directive::Strict { name, review },
                    "rank" => Directive::Rank {
                        name,
                        value: NumberLiteral(self.text(self.child(value_id, "signedNumber")?)),
                        review,
                    },
                    "contrary" => Directive::Contrary {
                        name,
                        other: self.text(names[1]),
                        review,
                    },
                    "prefer" => Directive::Prefer {
                        name,
                        other: self.text(names[1]),
                        review,
                    },
                    _ => {
                        return Err(self.error(
                            value_id,
                            "SYNTAX_TREE",
                            "Unknown argumentation directive",
                        ));
                    }
                };
                DeclarationKind::Argumentation { directive }
            }
            other => {
                return Err(self.error(
                    value_id,
                    "SYNTAX_TREE",
                    format!("Unknown declaration rule {other}"),
                ));
            }
        };
        Ok(Declaration {
            span: self.syntax.node(value_id).span,
            value,
        })
    }

    fn reference(&self, id: NodeId) -> Result<Reference> {
        let name = self.name(id)?;
        Ok(match self.first_text(id).as_str() {
            "head" if self.has_token(id, "(") => Reference::Head(name),
            "tail" if self.has_token(id, "(") => Reference::Tail(name),
            _ => Reference::Name(name),
        })
    }

    fn references(&self, id: NodeId) -> Result<Vec<Reference>> {
        self.children(id, "reference")
            .into_iter()
            .map(|reference| self.reference(reference))
            .collect()
    }

    fn groups(&self, id: NodeId, group_rule: &str) -> Result<Vec<SupportGroup>> {
        let mut groups = Vec::new();
        let mut seen = BTreeSet::new();
        for group in self.children(id, group_rule) {
            let name = self.first_text(group);
            if !seen.insert(name.clone()) {
                return Err(self.error(
                    group,
                    "DUPLICATE_SUPPORT",
                    format!("Duplicate support group {name:?}"),
                ));
            }
            let kind = match name.as_str() {
                "evidence" => SupportKind::Evidence,
                "assumptions" => SupportKind::Assumptions,
                "premises" => SupportKind::Premises,
                _ => return Err(self.error(group, "SYNTAX_TREE", "Unknown support group kind")),
            };
            groups.push(SupportGroup {
                kind,
                references: self.references(group)?,
            });
        }
        Ok(groups)
    }

    fn flow(&self, id: NodeId) -> Result<ArgumentFlow> {
        let references = self.references(id)?;
        Ok(ArgumentFlow {
            support: self.groups(self.child(id, "support")?, "supportGroup")?,
            reasoning: references[0].clone(),
            conclusion: references[1].clone(),
            binding: references.get(2).cloned(),
        })
    }
}
