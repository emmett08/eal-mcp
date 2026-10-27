//! Print checked EAL source and verify its source-independent composed meaning.
use crate::semantic as s;
use crate::{CoreError, ExecutionLimits, composition, expressions, methods, validation};
use eal_parser::SourceSpan;
use serde_json::Value;

type Result<T> = std::result::Result<T, CoreError>;

/// Interchange view excluding byte identity, locations and argument provenance.
/// Formatting changes the source digest and therefore requires newly bound records.
pub fn semantic_ir(program: &s::Program) -> Result<Value> {
    program
        .limits
        .validate()
        .map_err(|error| fail("FORMAT_RESOURCE", error))?;
    validation::structural_value_count(program, program.limits.structural_values)
        .map_err(|error| fail("FORMAT_RESOURCE", error))?;
    std::thread::scope(|scope| {
        std::thread::Builder::new()
            .name("eal-semantic-view".into())
            .stack_size(16 * 1024 * 1024)
            .spawn_scoped(scope, || semantic_view(program))
            .map_err(|error| {
                fail(
                    "FORMAT_RESOURCE",
                    format!("Cannot start semantic view worker: {error}"),
                )
            })?
            .join()
            .map_err(|_| fail("FORMAT_RESOURCE", "Semantic view worker failed"))?
    })
}

/// Fingerprint the checked executable meaning, independently of source bytes,
/// locations, import paths and retained authoring syntax. This is supplementary
/// identity: it never substitutes for the source digest on historical records.
pub fn executable_digest(
    program: &s::Program,
    registry: &methods::MethodRegistry,
) -> Result<String> {
    check(program, registry)?;
    let full = semantic_ir(program)?;
    let mut executable = serde_json::Map::new();
    executable.insert(
        "schema".into(),
        Value::String("EAL/executable-program/1".into()),
    );
    for key in [
        "language",
        "environments",
        "tools",
        "evidence",
        "assumptions",
        "reasoning",
        "claims",
        "arguments",
        "objections",
        "formal",
    ] {
        if let Some(value) = full.get(key) {
            executable.insert(key.into(), value.clone());
        }
    }
    if let Some(Value::Array(directives)) = executable.get_mut("formal") {
        for directive in directives {
            if let Some(fields) = directive.as_object_mut() {
                fields.remove("source_file");
            }
        }
    }
    crate::canonical::canonical_digest(&Value::Object(executable), &program.limits)
        .map_err(|error| fail("EXECUTABLE_DIGEST", error))
}

fn semantic_view(program: &s::Program) -> Result<Value> {
    let mut value = serde_json::to_value(program)
        .map_err(|error| fail("FORMAT_INVALID_IR", error.to_string()))?;
    let object = value
        .as_object_mut()
        .ok_or_else(|| fail("FORMAT_INVALID_IR", "Semantic program must be an object"))?;
    if let Some(directives) = object.remove("argumentation_directives") {
        object.insert("formal".into(), directives);
    }
    for name in ["source_digest", "locations", "authored", "limits"] {
        object.remove(name);
    }
    if let Some(Value::Object(arguments)) = object.get_mut("arguments") {
        for argument in arguments.values_mut() {
            if let Some(argument) = argument.as_object_mut() {
                argument.remove("origin");
            }
        }
    }
    for key in ["lowering_diagnostics", "formal"] {
        if let Some(Value::Array(entries)) = object.get_mut(key) {
            for entry in entries {
                if let Some(entry) = entry.as_object_mut() {
                    entry.remove("span");
                }
            }
        }
    }
    if let Some(Value::Object(patterns)) = object.get_mut("patterns") {
        for pattern in patterns.values_mut() {
            if let Some(body) = pattern.get_mut("body") {
                declaration_spans(body);
            }
        }
    }
    Ok(value)
}

// Only declaration locations are provenance. Payload fields named `span` carry
// user meaning and must remain part of the exact comparison.
fn declaration_spans(value: &mut Value) {
    let Some(declarations) = value.as_array_mut() else {
        return;
    };
    for declaration in declarations {
        let Some(declaration) = declaration.as_object_mut() else {
            continue;
        };
        declaration.remove("span");
        let Some(value) = declaration.get_mut("value") else {
            continue;
        };
        match value.get("kind").and_then(Value::as_str) {
            Some("module" | "context" | "argument" | "guard") => {
                if let Some(body) = value.get_mut("body") {
                    declaration_spans(body);
                }
                if let Some(otherwise) = value.get_mut("otherwise") {
                    declaration_spans(otherwise);
                }
            }
            Some("pattern") => {
                if let Some(body) = value.get_mut("body").and_then(|body| body.get_mut("value")) {
                    declaration_spans(body);
                }
            }
            _ => {}
        }
    }
}

fn fail(code: &'static str, message: impl Into<String>) -> CoreError {
    CoreError::Structure {
        code,
        message: message.into(),
        span: SourceSpan::default(),
    }
}
fn check(program: &s::Program, registry: &methods::MethodRegistry) -> Result<()> {
    let diagnostics = validation::validate(program, registry);
    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(fail(
            "INVALID_PROGRAM",
            diagnostics
                .iter()
                .map(|d| format!("{}: {}", d.code, d.message))
                .collect::<Vec<_>>()
                .join("; "),
        ))
    }
}

/// Format only a valid programme whose output preserves the entire semantic view.
/// Source imports are replayed from captured strings without calling external IO.
pub fn format_program(program: &s::Program, registry: &methods::MethodRegistry) -> Result<String> {
    program
        .limits
        .validate()
        .map_err(|error| fail("FORMAT_RESOURCE", error))?;
    validation::structural_value_count(program, program.limits.structural_values)
        .map_err(|error| fail("FORMAT_RESOURCE", error))?;
    std::thread::scope(|scope| {
        std::thread::Builder::new()
            .name("eal-semantic-format".into())
            .stack_size(16 * 1024 * 1024)
            .spawn_scoped(scope, || format_checked(program, registry))
            .map_err(|error| {
                fail(
                    "FORMAT_RESOURCE",
                    format!("Cannot start semantic format worker: {error}"),
                )
            })?
            .join()
            .map_err(|_| fail("FORMAT_RESOURCE", "Semantic format worker failed"))?
    })
}

fn format_checked(program: &s::Program, registry: &methods::MethodRegistry) -> Result<String> {
    check(program, registry)?;
    let authored = match &program.authored {
        Some(authored) => authored.clone(),
        None => direct_syntax(program)?,
    };
    let output = authored.format();
    let limits = eal_parser::ParseLimits {
        max_source_bytes: program.limits.source_bytes,
        max_tokens: program.limits.source_tokens,
        max_nesting: program.limits.expansion_depth,
        ..Default::default()
    };
    crate::parse_with_limits(&output, limits).map_err(|error| {
        fail(
            "FORMAT_RESOURCE",
            format!("Formatted source failed its input limits: {error}"),
        )
    })?;
    let resolver = |path: &str| {
        program
            .import_sources
            .get(path)
            .cloned()
            .ok_or_else(|| format!("No captured source for {path:?}"))
    };
    let reparsed = composition::compile(
        &output,
        Some(&resolver),
        &program.root_origin,
        program.limits.clone(),
    )?;
    check(&reparsed, registry)?;
    let before = semantic_view(program)?;
    let after = semantic_view(&reparsed)?;
    if before != after {
        let key = before
            .as_object()
            .expect("object")
            .keys()
            .find(|key| before[*key] != after[*key])
            .map_or("programme", String::as_str);
        return Err(fail(
            "FORMAT_SEMANTIC_CHANGE",
            format!("Formatted source changed semantic field {key:?}"),
        ));
    }
    if reparsed
        .authored
        .as_ref()
        .expect("compiled source retains syntax")
        .format()
        != output
    {
        return Err(fail(
            "FORMAT_NOT_IDEMPOTENT",
            "Formatting the reparsed source changed its canonical text",
        ));
    }
    Ok(output)
}

pub fn format_default(program: &s::Program) -> Result<String> {
    format_program(program, methods::default_registry())
}

/// Parse, validate and format source using an explicitly selected pure method registry.
pub fn format_source(
    source: &str,
    resolver: Option<&composition::SourceResolver<'_>>,
    origin: &str,
    limits: ExecutionLimits,
    registry: &methods::MethodRegistry,
) -> Result<String> {
    format_program(
        &composition::compile(source, resolver, origin, limits)?,
        registry,
    )
}

/// Plain direct IR has a faithful textual form without reconstructing source namespaces.
/// Template and import provenance requires the retained authored representation.
fn direct_syntax(program: &s::Program) -> Result<crate::Program> {
    if !program.patterns.is_empty()
        || !program.applications.is_empty()
        || !program.imports.is_empty()
        || !program.generated.is_empty()
    {
        return Err(fail(
            "FORMAT_AUTHORED_REQUIRED",
            "Direct IR containing templates, imports or generated declarations requires retained authored syntax for exact semantic formatting",
        ));
    }
    use crate::{
        Declaration, DeclarationKind as D, Field as F, JsonProperty as J, NameProperty as N,
        Reference as R, TextProperty as T,
    };
    let mut declarations = Vec::new();
    let mut push = |value| {
        declarations.push(Declaration {
            span: SourceSpan::default(),
            value,
        })
    };
    let text = |property, value: &str| F::Text {
        property,
        value: value.into(),
    };
    let name = |property, value: &str| F::Name {
        property,
        value: value.into(),
    };
    let predicates = |values: &[s::Predicate]| {
        values
            .iter()
            .map(|p| F::Require {
                expression: expressions::predicate_expression(p),
            })
            .collect::<Vec<_>>()
    };
    for value in program.environments.values() {
        push(D::Environment {
            name: value.name.clone(),
            predicates: value
                .predicates
                .iter()
                .map(expressions::predicate_expression)
                .collect(),
        });
    }
    for value in program.tools.values() {
        push(D::Tool {
            name: value.name.clone(),
            fields: vec![text(T::Version, &value.version)],
        });
    }
    for value in program.evidence.values() {
        let mut fields = vec![
            F::Kind {
                value: value.kind.clone(),
            },
            name(N::Environment, &value.environment),
            F::MaxAge {
                value: crate::NumberLiteral(crate::canonical::python_float_repr(value.max_age)),
            },
            F::Json {
                property: J::Input,
                value: expressions::json_to_literal(&value.input),
            },
        ];
        if let Some(tool) = &value.tool {
            fields.insert(0, name(N::Tool, tool));
        }
        if let Some(expression) = &value.derivation {
            fields.insert(
                0,
                F::Derive {
                    expression: expression.clone(),
                },
            );
        }
        fields.extend(value.bindings.iter().map(|binding| F::InputBinding {
            target: binding.target.clone(),
            evidence: binding.evidence.clone(),
            source: binding.source.clone(),
        }));
        fields.extend(predicates(&value.predicates));
        push(D::Evidence {
            name: value.name.clone(),
            fields,
        });
    }
    for value in program.assumptions.values() {
        let mut fields = vec![
            text(T::Statement, &value.statement),
            name(N::Environment, &value.environment),
            name(N::Validate, &value.validation),
        ];
        if let Some(date) = &value.valid_from {
            fields.push(text(T::ValidFrom, date));
        }
        if let Some(date) = &value.valid_until {
            fields.push(text(T::ValidUntil, date));
        }
        push(D::Assumption {
            name: value.name.clone(),
            fields,
        });
    }
    for value in program.reasoning.values() {
        let mut fields = vec![
            text(T::Method, &value.method),
            text(T::Rationale, &value.rationale),
        ];
        if !value.backing.is_empty() {
            fields.push(F::Backing {
                references: value.backing.iter().cloned().map(R::Name).collect(),
            });
        }
        if let Some(transfer) = &value.transfer {
            fields.push(F::Transfer {
                source: transfer.source.clone(),
                target: transfer.target.clone(),
                assumption: transfer.assumption.clone(),
                review: transfer.review.clone(),
            });
        }
        fields.extend(predicates(&value.predicates));
        push(D::Reasoning {
            name: value.name.clone(),
            fields,
        });
    }
    for value in program.claims.values() {
        let mut fields = vec![
            text(T::Statement, &value.statement),
            name(N::Environment, &value.environment),
        ];
        if let Some(p) = &value.proposition {
            fields.push(F::Proposition {
                fields: vec![
                    text(T::Subject, &p.subject),
                    text(T::Quantity, &p.quantity),
                    text(T::Unit, &p.unit),
                    text(T::Scope, &p.scope),
                    text(T::ValidFrom, &p.valid_from),
                    text(T::ValidUntil, &p.valid_until),
                    F::Json {
                        property: J::Query,
                        value: expressions::json_to_literal(&p.query),
                    },
                    F::Result {
                        expression: expressions::predicate_expression(&p.result),
                    },
                ],
            });
        }
        push(D::Claim {
            name: value.name.clone(),
            fields,
        });
    }
    let support = |evidence: &[String], assumptions: &[String], premises: &[String]| {
        [
            (crate::SupportKind::Evidence, evidence),
            (crate::SupportKind::Assumptions, assumptions),
            (crate::SupportKind::Premises, premises),
        ]
        .into_iter()
        .filter(|(_, v)| !v.is_empty())
        .map(|(kind, v)| crate::SupportGroup {
            kind,
            references: v.iter().cloned().map(R::Name).collect(),
        })
        .collect()
    };
    for value in program.arguments.values() {
        if value.origin.is_some() {
            return Err(fail(
                "FORMAT_AUTHORED_REQUIRED",
                "Generated arguments require retained authored syntax",
            ));
        }
        push(D::Argument {
            name: value.name.clone(),
            body: None,
            flow: crate::ArgumentFlow {
                support: support(&value.evidence, &value.assumptions, &value.premises),
                reasoning: R::Name(value.reasoning.clone()),
                conclusion: R::Name(value.conclusion.clone()),
                binding: value.binding.clone().map(R::Name),
            },
        });
    }
    for value in program.objections.values() {
        let target_kind = match value.target_kind.as_str() {
            "claim" => crate::TargetKind::Claim,
            "reasoning" => crate::TargetKind::Reasoning,
            "assumption" => crate::TargetKind::Assumption,
            "argument" => crate::TargetKind::Argument,
            "objection" => crate::TargetKind::Objection,
            _ => return Err(fail("INVALID_PROGRAM", "Unknown objection target kind")),
        };
        push(D::Objection {
            name: value.name.clone(),
            support: support(&value.evidence, &[], &value.premises),
            target_kind,
            target: R::Name(value.target.clone()),
        });
    }
    for value in &program.argumentation_directives {
        let directive = match value.kind.as_str() {
            "strict" => crate::Directive::Strict {
                name: value.name.clone(),
                review: value.review.clone(),
            },
            "rank" => crate::Directive::Rank {
                name: value.name.clone(),
                value: crate::NumberLiteral(
                    value
                        .rank
                        .ok_or_else(|| {
                            fail("INVALID_PROGRAM", "Rank directive requires an integer")
                        })?
                        .to_string(),
                ),
                review: value.review.clone(),
            },
            "contrary" => crate::Directive::Contrary {
                name: value.name.clone(),
                other: value.other.clone().ok_or_else(|| {
                    fail("INVALID_PROGRAM", "Contrary directive requires a target")
                })?,
                review: value.review.clone(),
            },
            "prefer" => crate::Directive::Prefer {
                name: value.name.clone(),
                other: value.other.clone().ok_or_else(|| {
                    fail("INVALID_PROGRAM", "Preference directive requires a target")
                })?,
                review: value.review.clone(),
            },
            _ => return Err(fail("INVALID_PROGRAM", "Unknown formal directive")),
        };
        push(D::Argumentation { directive });
    }
    Ok(crate::Program {
        language: program.language.clone(),
        declarations,
    })
}
