//! Check closed template references and finite, guarded recursive expansion.
use super::*;

fn declaration_name(value: &DeclarationKind) -> Option<(&str, &str)> {
    Some(match value {
        DeclarationKind::Environment { name, .. } => (name, "environment"),
        DeclarationKind::Tool { name, .. } => (name, "tool"),
        DeclarationKind::Evidence { name, .. } => (name, "evidence"),
        DeclarationKind::Assumption { name, .. } => (name, "assumption"),
        DeclarationKind::Reasoning { name, .. } => (name, "reasoning"),
        DeclarationKind::Claim { name, .. } => (name, "claim"),
        DeclarationKind::Argument { name, .. } => (name, "argument"),
        DeclarationKind::Objection { name, .. } => (name, "objection"),
        DeclarationKind::Pattern { name, .. } => (name, "pattern"),
        DeclarationKind::Application { name, .. } => (name, "application"),
        _ => return None,
    })
}

fn locals(declarations: &[Declaration], prefix: &str, result: &mut BTreeMap<String, String>) {
    for declaration in declarations {
        match &declaration.value {
            DeclarationKind::Context { body, .. } => locals(body, prefix, result),
            DeclarationKind::Module { name, body } => locals(body, &qualify(prefix, name), result),
            DeclarationKind::Argument {
                name,
                body: Some(body),
                ..
            } => {
                let child = qualify(prefix, name);
                locals(body, &child, result);
                result.insert(child, "argument".into());
            }
            DeclarationKind::Guard {
                body, otherwise, ..
            } => {
                locals(body, prefix, result);
                if let Some(otherwise) = otherwise {
                    locals(otherwise, prefix, result);
                }
            }
            _ => {
                if let Some((name, kind)) = declaration_name(&declaration.value) {
                    result.insert(qualify(prefix, name), kind.into());
                }
            }
        }
    }
}
fn pattern_definitions(
    declarations: &[Declaration],
    prefix: &str,
    defaults: &BTreeMap<String, Field>,
    result: &mut BTreeMap<String, s::Pattern>,
) {
    for declaration in declarations {
        match &declaration.value {
            DeclarationKind::Pattern {
                name,
                parameters,
                decreases,
                body,
            } => {
                result.insert(
                    qualify(prefix, name),
                    values::pattern(name, parameters, decreases, body, defaults),
                );
            }
            DeclarationKind::Context {
                defaults: local,
                body,
            } => {
                let mut inherited = defaults.clone();
                for field in local {
                    inherited.insert(field.name().into(), field.clone());
                }
                pattern_definitions(body, prefix, &inherited, result);
            }
            DeclarationKind::Module { name, body }
            | DeclarationKind::Argument {
                name,
                body: Some(body),
                ..
            } => pattern_definitions(body, &qualify(prefix, name), defaults, result),
            DeclarationKind::Guard {
                body, otherwise, ..
            } => {
                pattern_definitions(body, prefix, defaults, result);
                if let Some(otherwise) = otherwise {
                    pattern_definitions(otherwise, prefix, defaults, result);
                }
            }
            _ => {}
        }
    }
}
fn application_kinds(
    lowerer: &Lowerer<'_>,
    record: &Record,
    declarations: &[Declaration],
    prefix: &str,
    local_patterns: &BTreeMap<String, s::Pattern>,
    result: &mut BTreeMap<String, String>,
) {
    for declaration in declarations {
        match &declaration.value {
            DeclarationKind::Application { name, pattern, .. } => {
                let local = candidates(prefix, pattern)
                    .into_iter()
                    .find_map(|n| local_patterns.get(&n));
                let global = candidates(&record.scope.prefix, pattern)
                    .into_iter()
                    .find_map(|n| lowerer.records["pattern"].get(&n))
                    .and_then(|r| {
                        if let Entity::Pattern(p) = &r.value {
                            Some(p)
                        } else {
                            None
                        }
                    });
                if local.or(global).is_some_and(|p| p.body.is_none()) {
                    result.insert(qualify(prefix, name), "argument".into());
                }
            }
            DeclarationKind::Context { body, .. } => {
                application_kinds(lowerer, record, body, prefix, local_patterns, result)
            }
            DeclarationKind::Module { name, body }
            | DeclarationKind::Argument {
                name,
                body: Some(body),
                ..
            } => application_kinds(
                lowerer,
                record,
                body,
                &qualify(prefix, name),
                local_patterns,
                result,
            ),
            DeclarationKind::Guard {
                body, otherwise, ..
            } => {
                application_kinds(lowerer, record, body, prefix, local_patterns, result);
                if let Some(otherwise) = otherwise {
                    application_kinds(lowerer, record, otherwise, prefix, local_patterns, result);
                }
            }
            _ => {}
        }
    }
}

struct Template<'a> {
    pattern: &'a s::Pattern,
    record: &'a Record,
    parameters: BTreeMap<String, String>,
    locals: BTreeMap<String, String>,
    local_patterns: BTreeMap<String, s::Pattern>,
}
impl Lowerer<'_> {
    pub(super) fn check_template(&mut self, record: &Record) -> Result<()> {
        let Entity::Pattern(original) = &record.value else {
            return Ok(());
        };
        let mut pattern = original.clone();
        if let Some(body) = &pattern.body {
            pattern.body = Some(
                self.template_imports(body, record.scope.source_file.as_deref().unwrap_or(""))?,
            );
        }
        let pattern = &pattern;
        if !self.checked.insert(pattern.name.clone()) {
            return Ok(());
        }
        let mut parameters = BTreeMap::new();
        for parameter in &pattern.parameters {
            if parameters
                .insert(parameter.name.clone(), parameter.kind.clone())
                .is_some()
            {
                self.error(
                    "duplicate_pattern_parameter",
                    format!(
                        "Parameter {} is declared more than once",
                        repr(&parameter.name)
                    ),
                    Some(&pattern.name),
                    None,
                    None,
                    None,
                );
            }
            if !matches!(
                parameter.kind.trim_end_matches("[]"),
                "claim" | "reasoning" | "evidence" | "assumption" | "environment" | "tool"
            ) {
                self.error(
                    "invalid_pattern_parameter_kind",
                    format!("Unknown parameter kind {}", repr(&parameter.kind)),
                    Some(&pattern.name),
                    None,
                    None,
                    None,
                );
            }
        }
        if let Some(decreasing) = &pattern.decreases
            && !parameters
                .get(decreasing)
                .is_some_and(|p| p.ends_with("[]"))
        {
            self.error(
                "invalid_recursion_measure",
                "decreases must name a typed list parameter",
                Some(&pattern.name),
                None,
                None,
                None,
            );
        }
        let mut local = BTreeMap::new();
        let mut local_patterns = BTreeMap::new();
        if let Some(body) = &pattern.body {
            locals(body, "", &mut local);
            pattern_definitions(body, "", &pattern.defaults, &mut local_patterns);
            application_kinds(self, record, body, "", &local_patterns, &mut local);
        }
        for name in local.keys() {
            if parameters.contains_key(name) {
                self.error(
                    "generated_symbol_collision",
                    format!(
                        "Local {name_repr} shadows a pattern parameter",
                        name_repr = repr(name)
                    ),
                    Some(&pattern.name),
                    None,
                    None,
                    None,
                );
            }
        }
        let context = Template {
            pattern,
            record,
            parameters,
            locals: local,
            local_patterns,
        };
        if let Some(body) = &pattern.body {
            self.check_body(body, "", &BTreeSet::new(), &pattern.defaults, &context)?;
        } else {
            if pattern.evidence.is_empty()
                && pattern.assumptions.is_empty()
                && pattern.premises.is_empty()
            {
                self.error(
                    "empty_pattern",
                    "An argument pattern requires support",
                    Some(&pattern.name),
                    None,
                    None,
                    None,
                );
            }
            self.check_references(&record.value, "", &context);
        }
        Ok(())
    }
    fn check_reference(
        &mut self,
        reference: &str,
        kind: &str,
        many: bool,
        prefix: &str,
        context: &Template<'_>,
    ) {
        let op = operation(reference);
        let name = op.map_or(reference, |(_, name)| name);
        let actual = if let Some(actual) = context.parameters.get(name) {
            if let Some((op, _)) = op {
                if !actual.ends_with("[]") {
                    self.error(
                        "pattern_reference_kind",
                        format!("{op} requires a list parameter"),
                        Some(&context.pattern.name),
                        None,
                        None,
                        None,
                    );
                    return;
                }
                Some(if op == "tail" {
                    actual.clone()
                } else {
                    actual.trim_end_matches("[]").into()
                })
            } else {
                Some(actual.clone())
            }
        } else {
            candidates(prefix, name)
                .into_iter()
                .find_map(|name| context.locals.get(&name).cloned())
        };
        match actual{
            None=>self.error("unbound_pattern_reference",format!("Reference {name_repr} must name a typed parameter or local declaration; global capture is forbidden", name_repr=repr(name)),Some(&context.pattern.name),None,Some(kind),Some("undeclared parameter")),
            Some(actual) if actual!=kind&&!(many&&actual==format!("{kind}[]"))=>self.error("pattern_reference_kind",format!("Reference {name_repr} has kind {actual_repr}; expected {kind_repr}", name_repr=repr(name), actual_repr=repr(&actual), kind_repr=repr(kind)),Some(&context.pattern.name),None,Some(kind),Some(&actual)),_=>{}
        }
    }
    fn check_references(&mut self, entity: &Entity, prefix: &str, context: &Template<'_>) {
        for (reference, kind, many) in entity.references() {
            self.check_reference(&reference, &kind, many, prefix, context);
        }
    }
    fn check_body(
        &mut self,
        declarations: &[Declaration],
        prefix: &str,
        guards: &BTreeSet<String>,
        defaults: &BTreeMap<String, Field>,
        context: &Template<'_>,
    ) -> Result<()> {
        for declaration in declarations {
            match &declaration.value {
                DeclarationKind::Context {
                    defaults: local,
                    body,
                } => {
                    let mut inherited = defaults.clone();
                    for field in local {
                        inherited.insert(field.name().into(), field.clone());
                    }
                    self.check_body(body, prefix, guards, &inherited, context)?;
                }
                DeclarationKind::Module { name, body } => {
                    self.check_body(body, &qualify(prefix, name), guards, defaults, context)?
                }
                DeclarationKind::Argument {
                    name,
                    body: Some(body),
                    flow,
                } => {
                    let child = qualify(prefix, name);
                    self.check_body(body, &child, guards, defaults, context)?;
                    self.check_references(
                        &Entity::Argument(values::argument(name, flow)),
                        &child,
                        context,
                    );
                }
                DeclarationKind::Guard {
                    parameter,
                    body,
                    otherwise,
                } => {
                    if !context
                        .parameters
                        .get(parameter)
                        .is_some_and(|p| p.ends_with("[]"))
                    {
                        self.error(
                            "pattern_guard_type",
                            "when requires a typed list parameter",
                            Some(&context.pattern.name),
                            None,
                            None,
                            None,
                        );
                    }
                    let mut guarded = guards.clone();
                    guarded.insert(parameter.clone());
                    self.check_body(body, prefix, &guarded, defaults, context)?;
                    if let Some(otherwise) = otherwise {
                        self.check_body(otherwise, prefix, guards, defaults, context)?;
                    }
                }
                DeclarationKind::Pattern {
                    name,
                    parameters,
                    decreases,
                    body,
                } => {
                    let mut nested = values::pattern(
                        &qualify(&context.pattern.name, name),
                        parameters,
                        decreases,
                        body,
                        defaults,
                    );
                    nested.defaults = defaults.clone();
                    let scope = Scope {
                        prefix: context.record.scope.prefix.clone(),
                        span: declaration.span,
                        ..Scope::root("")
                    };
                    self.check_template(&Record {
                        value: Entity::Pattern(nested),
                        scope,
                    })?;
                }
                DeclarationKind::Application {
                    name: _,
                    pattern,
                    bindings,
                } => {
                    let global = candidates(&context.record.scope.prefix, pattern)
                        .into_iter()
                        .find_map(|n| self.records["pattern"].get(&n))
                        .and_then(|r| {
                            if let Entity::Pattern(p) = &r.value {
                                Some(p.clone())
                            } else {
                                None
                            }
                        });
                    let local = candidates(prefix, pattern)
                        .into_iter()
                        .find_map(|n| context.local_patterns.get(&n).map(|p| (n, p.clone())));
                    let called = if let Some((name, mut pattern)) = local {
                        pattern.name = qualify(&context.pattern.name, &name);
                        Some(pattern)
                    } else if pattern == &context.pattern.name
                        || pattern == context.pattern.name.rsplit('.').next().unwrap_or("")
                    {
                        Some(context.pattern.clone())
                    } else {
                        global
                    };
                    if let Some(called) = called {
                        let supplied: BTreeMap<_, _> = bindings
                            .iter()
                            .map(|binding| {
                                (
                                    binding.name.clone(),
                                    match &binding.value {
                                        crate::BindingValue::Single(r) => {
                                            s::BindingValue::Single(values::reference(r))
                                        }
                                        crate::BindingValue::List(r) => s::BindingValue::List(
                                            r.iter().map(values::reference).collect(),
                                        ),
                                    },
                                )
                            })
                            .collect();
                        let kinds: BTreeMap<_, _> = called
                            .parameters
                            .iter()
                            .map(|p| (p.name.clone(), p.kind.clone()))
                            .collect();
                        if supplied.len() != bindings.len() {
                            self.error(
                                "duplicate_pattern_argument",
                                "A parameter is supplied more than once",
                                Some(&context.pattern.name),
                                None,
                                None,
                                None,
                            );
                        }
                        for binding in bindings {
                            let Some(expected) = kinds.get(&binding.name) else {
                                self.error(
                                    "unknown_pattern_argument",
                                    format!("Unknown parameter {}", repr(&binding.name)),
                                    Some(&context.pattern.name),
                                    None,
                                    None,
                                    None,
                                );
                                continue;
                            };
                            let refs: Vec<String> = match &binding.value {
                                crate::BindingValue::Single(r) => vec![values::reference(r)],
                                crate::BindingValue::List(r) => {
                                    r.iter().map(values::reference).collect()
                                }
                            };
                            if matches!(binding.value, crate::BindingValue::List(_))
                                && !expected.ends_with("[]")
                            {
                                self.error(
                                    "pattern_argument_kind",
                                    format!(
                                        "Parameter {} requires a scalar reference",
                                        repr(&binding.name)
                                    ),
                                    Some(&context.pattern.name),
                                    None,
                                    None,
                                    None,
                                );
                            }
                            if let crate::BindingValue::Single(reference) = &binding.value
                                && expected.ends_with("[]")
                            {
                                let reference = values::reference(reference);
                                let op = operation(&reference);
                                let actual = context
                                    .parameters
                                    .get(op.map_or(reference.as_str(), |(_, n)| n));
                                if !actual.is_some_and(|p| p.ends_with("[]"))
                                    || op.is_some_and(|(op, _)| op == "head")
                                {
                                    self.error("pattern_argument_kind",format!("Parameter {} requires a list reference or explicit list", repr(&binding.name)),Some(&context.pattern.name),None,None,None);
                                }
                            }
                            for reference in refs {
                                self.check_reference(
                                    &reference,
                                    expected.trim_end_matches("[]"),
                                    expected.ends_with("[]"),
                                    prefix,
                                    context,
                                );
                            }
                        }
                        for missing in kinds.keys().filter(|name| !supplied.contains_key(*name)) {
                            self.error(
                                "missing_pattern_argument",
                                format!(
                                    "Pattern {} requires {missing_repr}",
                                    repr(&called.name),
                                    missing_repr = repr(missing)
                                ),
                                Some(&context.pattern.name),
                                None,
                                None,
                                None,
                            );
                        }
                        self.calls.push(Call {
                            caller: context.pattern.name.clone(),
                            called: called.name.clone(),
                            supplied: supplied.clone(),
                            guards: guards.clone(),
                        });
                        if called.name == context.pattern.name {
                            let decreasing = context.pattern.decreases.as_deref();
                            if !decreasing.is_some_and(|d| {
                                guards.contains(d)
                                    && supplied.get(d)
                                        == Some(&s::BindingValue::Single(format!("tail({d})")))
                            }) {
                                self.error("nonterminating_pattern","A recursive call must be guarded by the decreasing list and pass its tail",Some(&context.pattern.name),None,None,None);
                            }
                        }
                    } else if !context.locals.contains_key(pattern) {
                        self.error(
                            "unknown_pattern",
                            format!(
                                "Unknown pattern {pattern_repr}",
                                pattern_repr = repr(pattern)
                            ),
                            Some(&context.pattern.name),
                            None,
                            None,
                            None,
                        );
                    }
                }
                DeclarationKind::Argumentation { .. } => {}
                DeclarationKind::Import { .. } => { /* Imported bodies are checked after trusted source expansion. */
                }
                _ => {
                    self.check_references(&values::entity(declaration, defaults)?, prefix, context)
                }
            }
        }
        Ok(())
    }
    pub(super) fn check_recursive_components(&mut self) {
        fn reaches(start: &str, goal: &str, graph: &BTreeMap<String, BTreeSet<String>>) -> bool {
            let mut pending = vec![start.to_owned()];
            let mut seen = BTreeSet::new();
            while let Some(node) = pending.pop() {
                if node == goal {
                    return true;
                }
                if seen.insert(node.clone())
                    && let Some(edges) = graph.get(&node)
                {
                    pending.extend(edges.iter().cloned());
                }
            }
            false
        }
        let mut graph: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        for call in &self.calls {
            graph
                .entry(call.caller.clone())
                .or_default()
                .insert(call.called.clone());
        }
        for call in self.calls.clone() {
            if reaches(&call.called, &call.caller, &graph) {
                let source = self.records["pattern"].get(&call.caller).and_then(|r| {
                    if let Entity::Pattern(p) = &r.value {
                        Some(p)
                    } else {
                        None
                    }
                });
                let target = self.records["pattern"].get(&call.called).and_then(|r| {
                    if let Entity::Pattern(p) = &r.value {
                        Some(p)
                    } else {
                        None
                    }
                });
                if let (Some(source), Some(target)) = (source, target) {
                    let valid = source
                        .decreases
                        .as_ref()
                        .zip(target.decreases.as_ref())
                        .is_some_and(|(a, b)| {
                            call.guards.contains(a)
                                && call.supplied.get(b)
                                    == Some(&s::BindingValue::Single(format!("tail({a})")))
                        });
                    if !valid {
                        self.error("nonterminating_pattern","Every call in a recursive component must pass a guarded decreasing tail",Some(&call.caller),None,None,None);
                    }
                }
            }
        }
    }
}
