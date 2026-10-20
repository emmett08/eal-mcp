//! Lower JSON and expression syntax without selecting evaluation semantics.
use super::*;

impl Builder<'_> {
    pub(super) fn json(&self, id: NodeId) -> Result<JsonValue> {
        match self.syntax.node(id).rule_name().unwrap_or("") {
            "jsonValue" => self.json(
                self.syntax
                    .children(id)
                    .next()
                    .ok_or_else(|| self.error(id, "SYNTAX_TREE", "Empty JSON value"))?,
            ),
            "jsonScalar" => {
                if let Ok(string) = self.string_token(id) {
                    return Ok(JsonValue::String(self.decode_string(string)?));
                }
                Ok(match self.text(id).as_str() {
                    "null" => JsonValue::Null,
                    "true" => JsonValue::Boolean(true),
                    "false" => JsonValue::Boolean(false),
                    _ => JsonValue::Number(NumberLiteral(self.text(id))),
                })
            }
            "jsonArray" => Ok(JsonValue::Array(
                self.children(id, "jsonValue")
                    .into_iter()
                    .map(|child| self.json(child))
                    .collect::<Result<_>>()?,
            )),
            "jsonObject" => {
                let keys = self.syntax.children(id).filter(|&child| {
                    self.syntax
                        .token(child)
                        .is_some_and(|token| token.kind == "STRING")
                });
                let mut seen = BTreeSet::new();
                let mut entries = Vec::new();
                for (key_id, value_id) in keys.zip(self.children(id, "jsonValue")) {
                    let key = self.decode_string(key_id)?;
                    if !seen.insert(key.clone()) {
                        return Err(self.error(
                            key_id,
                            "DUPLICATE_JSON_KEY",
                            format!("Duplicate JSON object key {key:?}"),
                        ));
                    }
                    entries.push((key, self.json(value_id)?));
                }
                Ok(JsonValue::Object(entries))
            }
            other => Err(self.error(id, "SYNTAX_TREE", format!("Unknown JSON rule {other}"))),
        }
    }

    fn check_expression(&self, id: NodeId, expression: &Expression) -> Result<()> {
        let mut pending = vec![(expression, 1_usize)];
        while let Some((expression, depth)) = pending.pop() {
            if depth > MAX_EXPRESSION_DEPTH {
                return Err(self.error(
                    id,
                    "EXPRESSION_DEPTH",
                    format!("Typed expression exceeds {MAX_EXPRESSION_DEPTH} levels"),
                ));
            }
            match expression {
                Expression::Unary { operand, .. } => pending.push((operand, depth + 1)),
                Expression::Binary { left, right, .. } => {
                    pending.push((left, depth + 1));
                    pending.push((right, depth + 1));
                }
                Expression::Call { arguments, .. } => {
                    pending.extend(arguments.iter().map(|argument| (argument, depth + 1)))
                }
                _ => {}
            }
        }
        Ok(())
    }

    pub(super) fn expression(&self, id: NodeId) -> Result<Expression> {
        let expression = self.expression_node(id)?;
        self.check_expression(id, &expression)?;
        Ok(expression)
    }

    fn expression_node(&self, id: NodeId) -> Result<Expression> {
        let rule = self.syntax.node(id).rule_name().unwrap_or("");
        match rule {
            "expression" => self.expression(self.child(id, "orExpression")?),
            "orExpression"
            | "andExpression"
            | "comparisonExpression"
            | "additiveExpression"
            | "multiplicativeExpression" => {
                let children: Vec<_> = self.syntax.children(id).collect();
                let mut result = self.expression(children[0])?;
                for pair in children[1..].chunks_exact(2) {
                    let operator = match self.text(pair[0]).as_str() {
                        "or" => BinaryOperator::Or,
                        "and" => BinaryOperator::And,
                        "==" => BinaryOperator::Equal,
                        "!=" => BinaryOperator::NotEqual,
                        "<=" => BinaryOperator::LessEqual,
                        ">=" => BinaryOperator::GreaterEqual,
                        "<" => BinaryOperator::Less,
                        ">" => BinaryOperator::Greater,
                        "+" => BinaryOperator::Add,
                        "-" => BinaryOperator::Subtract,
                        "*" => BinaryOperator::Multiply,
                        "/" => BinaryOperator::Divide,
                        "%" => BinaryOperator::Modulo,
                        other => {
                            return Err(self.error(
                                pair[0],
                                "SYNTAX_TREE",
                                format!("Unknown expression operator {other}"),
                            ));
                        }
                    };
                    if matches!(
                        operator,
                        BinaryOperator::Equal
                            | BinaryOperator::NotEqual
                            | BinaryOperator::LessEqual
                            | BinaryOperator::GreaterEqual
                            | BinaryOperator::Less
                            | BinaryOperator::Greater
                    ) && let Expression::Literal {
                        value: JsonValue::String(name),
                    } = result
                    {
                        result = Expression::Field { name };
                    }
                    result = Expression::Binary {
                        operator,
                        left: Box::new(result),
                        right: Box::new(self.expression(pair[1])?),
                    };
                    self.check_expression(id, &result)?;
                }
                Ok(result)
            }
            "unaryExpression" => {
                if let Some(&child) = self.children(id, "unaryExpression").first() {
                    let operator = match self.first_text(id).as_str() {
                        "not" => UnaryOperator::Not,
                        "+" => UnaryOperator::Positive,
                        _ => UnaryOperator::Negative,
                    };
                    Ok(Expression::Unary {
                        operator,
                        operand: Box::new(self.expression(child)?),
                    })
                } else {
                    self.expression(self.child(id, "primaryExpression")?)
                }
            }
            "primaryExpression" => {
                if let Some(&name_id) = self.children(id, "qualifiedName").first() {
                    let name = self.text(name_id);
                    if !self.has_token(id, "(") {
                        return Ok(Expression::Field { name });
                    }
                    let arguments = self
                        .children(id, "expression")
                        .into_iter()
                        .map(|child| self.expression(child))
                        .collect::<Result<Vec<_>>>()?;
                    if name == "field" {
                        return match arguments.as_slice() {
                            [
                                Expression::Literal {
                                    value: JsonValue::String(name),
                                },
                            ] => Ok(Expression::Field { name: name.clone() }),
                            _ => Err(self.error(
                                id,
                                "FIELD_ARGUMENT",
                                "field requires one literal property path",
                            )),
                        };
                    }
                    return Ok(Expression::Call { name, arguments });
                }
                if let Some(&json) = self.children(id, "jsonValue").first() {
                    return Ok(Expression::Literal {
                        value: self.json(json)?,
                    });
                }
                self.expression(self.child(id, "expression")?)
            }
            other => Err(self.error(
                id,
                "SYNTAX_TREE",
                format!("Unknown expression rule {other}"),
            )),
        }
    }
}
