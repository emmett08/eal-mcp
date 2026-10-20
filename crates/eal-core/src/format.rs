//! Canonical source formatting of the typed, unexpanded authored syntax.
use std::fmt::Write;

use crate::ast::*;

impl Program {
    /// Format authored declarations, retaining their order and explicit context defaults.
    /// Comments and source positions are presentation data and are omitted.
    /// Output size and token limits are unchecked. Use [`Self::format_with_limits`]
    /// when the result must be accepted under an operator's parser limits.
    pub fn format(&self) -> String {
        let mut output = format!("language {}\n", quote(&self.language));
        for declaration in &self.declarations {
            declaration_source(&mut output, declaration, 0);
        }
        output
    }

    /// Return canonical source only when it parses within the supplied limits
    /// and preserves the typed authored syntax, excluding source positions.
    /// Indentation and explicit property-path calls can enlarge the output.
    pub fn format_with_limits(
        &self,
        limits: eal_parser::ParseLimits,
    ) -> Result<String, crate::CoreError> {
        let output = self.format();
        let reparsed = crate::parse_with_limits(&output, limits)?;
        if !self.equivalent_to(&reparsed) {
            return Err(crate::CoreError::Structure {
                code: "FORMAT_EQUIVALENCE",
                message: "Formatted source changed the typed authored syntax".to_owned(),
                span: eal_parser::SourceSpan::default(),
            });
        }
        Ok(output)
    }

    /// Compare typed authored meaning while ignoring source positions and comments.
    pub fn equivalent_to(&self, other: &Self) -> bool {
        let mut left = self.clone();
        let mut right = other.clone();
        erase_spans(&mut left.declarations);
        erase_spans(&mut right.declarations);
        left == right
    }
}

fn erase_spans(declarations: &mut [Declaration]) {
    for declaration in declarations {
        declaration.span = eal_parser::SourceSpan::default();
        match &mut declaration.value {
            DeclarationKind::Module { body, .. } | DeclarationKind::Context { body, .. } => {
                erase_spans(body)
            }
            DeclarationKind::Argument {
                body: Some(body), ..
            } => erase_spans(body),
            DeclarationKind::Pattern {
                body: PatternBody::Block(body),
                ..
            } => erase_spans(body),
            DeclarationKind::Guard {
                body, otherwise, ..
            } => {
                erase_spans(body);
                if let Some(otherwise) = otherwise {
                    erase_spans(otherwise);
                }
            }
            _ => {}
        }
    }
}

fn quote(value: &str) -> String {
    // A Rust string is Unicode scalar data and is always serialisable as JSON.
    serde_json::to_string(value).expect("serialising a Rust string is infallible")
}

fn json(value: &JsonValue) -> String {
    match value {
        JsonValue::Null => "null".to_owned(),
        JsonValue::Boolean(value) => value.to_string(),
        JsonValue::Number(value) => value.0.clone(),
        JsonValue::String(value) => quote(value),
        JsonValue::Array(values) => format!(
            "[{}]",
            values.iter().map(json).collect::<Vec<_>>().join(", ")
        ),
        JsonValue::Object(entries) => format!(
            "{{{}}}",
            entries
                .iter()
                .map(|(name, value)| format!("{}: {}", quote(name), json(value)))
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}

fn reference(value: &Reference) -> String {
    match value {
        Reference::Name(name) => name.clone(),
        Reference::Head(name) => format!("head({name})"),
        Reference::Tail(name) => format!("tail({name})"),
    }
}

fn references(values: &[Reference]) -> String {
    values.iter().map(reference).collect::<Vec<_>>().join(", ")
}

fn expression(value: &Expression) -> String {
    match value {
        Expression::Literal { value } => json(value),
        Expression::Field { name } => format!("field({})", quote(name)),
        Expression::Call { name, arguments } => format!(
            "{name}({})",
            arguments
                .iter()
                .map(expression)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        Expression::Unary { operator, operand } => {
            let symbol = match operator {
                UnaryOperator::Not => "not ",
                UnaryOperator::Positive => "+",
                UnaryOperator::Negative => "-",
            };
            format!("{symbol}{}", operand_source(operand, 6, false))
        }
        Expression::Binary {
            operator,
            left,
            right,
        } => {
            let symbol = match operator {
                BinaryOperator::Or => "or",
                BinaryOperator::And => "and",
                BinaryOperator::Equal => "==",
                BinaryOperator::NotEqual => "!=",
                BinaryOperator::LessEqual => "<=",
                BinaryOperator::GreaterEqual => ">=",
                BinaryOperator::Less => "<",
                BinaryOperator::Greater => ">",
                BinaryOperator::Add => "+",
                BinaryOperator::Subtract => "-",
                BinaryOperator::Multiply => "*",
                BinaryOperator::Divide => "/",
                BinaryOperator::Modulo => "%",
            };
            let precedence = precedence(value);
            let comparison = precedence == 3;
            format!(
                "{} {symbol} {}",
                operand_source(left, precedence, comparison),
                operand_source(right, precedence, true)
            )
        }
    }
}

fn precedence(value: &Expression) -> u8 {
    match value {
        Expression::Binary { operator, .. } => match operator {
            BinaryOperator::Or => 1,
            BinaryOperator::And => 2,
            BinaryOperator::Equal
            | BinaryOperator::NotEqual
            | BinaryOperator::LessEqual
            | BinaryOperator::GreaterEqual
            | BinaryOperator::Less
            | BinaryOperator::Greater => 3,
            BinaryOperator::Add | BinaryOperator::Subtract => 4,
            BinaryOperator::Multiply | BinaryOperator::Divide | BinaryOperator::Modulo => 5,
        },
        Expression::Unary { .. } => 6,
        _ => 7,
    }
}

fn operand_source(value: &Expression, parent_precedence: u8, parenthesise_equal: bool) -> String {
    let source = expression(value);
    let precedence = precedence(value);
    if precedence < parent_precedence || (parenthesise_equal && precedence == parent_precedence) {
        format!("({source})")
    } else {
        source
    }
}

fn field_source(value: &Field) -> String {
    let name = value.name();
    let content = match value {
        Field::Text { value, .. } => quote(value),
        Field::Name { value, .. } | Field::Kind { value } => value.clone(),
        Field::MaxAge { value } => value.0.clone(),
        Field::Json { value, .. } => json(value),
        Field::Require { expression: value } | Field::Result { expression: value } => {
            expression(value)
        }
        Field::Backing { references: values } => format!("[{}]", references(values)),
        Field::Transfer {
            source,
            target,
            assumption,
            review,
        } => format!(
            "from {source} to {target} assuming {assumption} reviewed {}",
            quote(review)
        ),
        Field::Proposition { .. } => unreachable!("propositions use block formatting"),
    };
    format!("{name} {content}")
}

fn indent(output: &mut String, depth: usize) {
    output.push_str(&"    ".repeat(depth));
}

fn fields_source(output: &mut String, fields: &[Field], depth: usize) {
    for field in fields {
        indent(output, depth);
        if let Field::Proposition { fields } = field {
            output.push_str("proposition {\n");
            fields_source(output, fields, depth + 1);
            indent(output, depth);
            output.push_str("}\n");
        } else {
            output.push_str(&field_source(field));
            output.push('\n');
        }
    }
}

fn support(values: &[SupportGroup]) -> String {
    format!(
        "[{}]",
        values
            .iter()
            .map(|value| {
                let kind = match value.kind {
                    SupportKind::Evidence => "evidence",
                    SupportKind::Assumptions => "assumptions",
                    SupportKind::Premises => "premises",
                };
                format!("{kind} {}", references(&value.references))
            })
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn flow(value: &ArgumentFlow) -> String {
    let mut source = format!(
        "{} via {} => {}",
        support(&value.support),
        reference(&value.reasoning),
        reference(&value.conclusion)
    );
    if let Some(binding) = &value.binding {
        write!(source, " binding {}", reference(binding)).expect("writing to String");
    }
    source
}

fn block_source(output: &mut String, body: &[Declaration], depth: usize) {
    output.push_str(" {\n");
    for declaration in body {
        declaration_source(output, declaration, depth + 1);
    }
    indent(output, depth);
    output.push_str("}\n");
}

fn named_fields(output: &mut String, kind: &str, name: &str, fields: &[Field], depth: usize) {
    writeln!(output, "{kind} {name} {{").expect("writing to String");
    fields_source(output, fields, depth + 1);
    indent(output, depth);
    output.push_str("}\n");
}

fn declaration_source(output: &mut String, declaration: &Declaration, depth: usize) {
    indent(output, depth);
    match &declaration.value {
        DeclarationKind::Module { name, body } => {
            write!(output, "module {name}").expect("writing to String");
            block_source(output, body, depth);
        }
        DeclarationKind::Import { path, alias } => {
            writeln!(output, "import {} as {alias}", quote(path)).expect("writing to String");
        }
        DeclarationKind::Context { defaults, body } => {
            write!(
                output,
                "context {}",
                defaults
                    .iter()
                    .map(field_source)
                    .collect::<Vec<_>>()
                    .join(", ")
            )
            .expect("writing to String");
            block_source(output, body, depth);
        }
        DeclarationKind::Environment { name, predicates } => {
            writeln!(output, "environment {name} {{").expect("writing to String");
            for predicate in predicates {
                indent(output, depth + 1);
                writeln!(output, "require {}", expression(predicate)).expect("writing to String");
            }
            indent(output, depth);
            output.push_str("}\n");
        }
        DeclarationKind::Tool { name, fields } => named_fields(output, "tool", name, fields, depth),
        DeclarationKind::Evidence { name, fields } => {
            named_fields(output, "evidence", name, fields, depth)
        }
        DeclarationKind::Assumption { name, fields } => {
            named_fields(output, "assumption", name, fields, depth)
        }
        DeclarationKind::Reasoning { name, fields } => {
            named_fields(output, "reasoning", name, fields, depth)
        }
        DeclarationKind::Claim { name, fields } => {
            named_fields(output, "claim", name, fields, depth)
        }
        DeclarationKind::Argument {
            name,
            body,
            flow: value,
        } => {
            if let Some(body) = body {
                writeln!(output, "argument {name} {{").expect("writing to String");
                for declaration in body {
                    declaration_source(output, declaration, depth + 1);
                }
                indent(output, depth + 1);
                writeln!(output, "{}", flow(value)).expect("writing to String");
                indent(output, depth);
                output.push_str("}\n");
            } else {
                writeln!(output, "argument {name} = {}", flow(value)).expect("writing to String");
            }
        }
        DeclarationKind::Pattern {
            name,
            parameters,
            decreases,
            body,
        } => {
            write!(
                output,
                "pattern {name}({})",
                parameters
                    .iter()
                    .map(|parameter| format!(
                        "{}: {}{}",
                        parameter.name,
                        parameter.kind,
                        if parameter.list { "[]" } else { "" }
                    ))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
            .expect("writing to String");
            if let Some(decreases) = decreases {
                write!(output, " decreases {decreases}").expect("writing to String");
            }
            match body {
                PatternBody::Flow(value) => {
                    writeln!(output, " = {}", flow(value)).expect("writing to String");
                }
                PatternBody::Block(body) => block_source(output, body, depth),
            }
        }
        DeclarationKind::Application {
            name,
            pattern,
            bindings,
        } => {
            writeln!(
                output,
                "apply {name} = {pattern}({})",
                bindings
                    .iter()
                    .map(|binding| {
                        let value = match &binding.value {
                            BindingValue::Single(value) => reference(value),
                            BindingValue::List(values) => format!("[{}]", references(values)),
                        };
                        format!("{} = {value}", binding.name)
                    })
                    .collect::<Vec<_>>()
                    .join(", ")
            )
            .expect("writing to String");
        }
        DeclarationKind::Guard {
            parameter,
            body,
            otherwise,
        } => {
            write!(output, "when {parameter}").expect("writing to String");
            block_source(output, body, depth);
            if let Some(otherwise) = otherwise {
                indent(output, depth);
                output.push_str("else");
                block_source(output, otherwise, depth);
            }
        }
        DeclarationKind::Objection {
            name,
            support: values,
            target_kind,
            target,
        } => {
            let kind = match target_kind {
                TargetKind::Claim => "claim",
                TargetKind::Reasoning => "reasoning",
                TargetKind::Assumption => "assumption",
                TargetKind::Argument => "argument",
                TargetKind::Objection => "objection",
            };
            writeln!(
                output,
                "objection {name} = {} -x> {kind} {}",
                support(values),
                reference(target)
            )
            .expect("writing to String");
        }
        DeclarationKind::Argumentation { directive } => {
            let source = match directive {
                Directive::Strict { name, review } => {
                    format!("strict {name} reviewed {}", quote(review))
                }
                Directive::Rank {
                    name,
                    value,
                    review,
                } => format!("rank {name} {} reviewed {}", value.0, quote(review)),
                Directive::Contrary {
                    name,
                    other,
                    review,
                } => format!("contrary {name} to {other} reviewed {}", quote(review)),
                Directive::Prefer {
                    name,
                    other,
                    review,
                } => format!("prefer {name} over {other} reviewed {}", quote(review)),
            };
            writeln!(output, "{source}").expect("writing to String");
        }
    }
}
