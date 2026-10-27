//! Bind lexical declarations and expand closed, typed argument patterns.
use crate::semantic as s;
use crate::{CoreError, Declaration, DeclarationKind, ExecutionLimits, Field};
use eal_parser::SourceSpan;
use indexmap::IndexMap;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

mod binding;
mod direct;
mod expansion;
mod imports;
mod records;
mod templates;
use records::*;
mod values;
pub use values::pattern_metadata;

pub type SourceResolver<'a> = dyn Fn(&str) -> std::result::Result<String, String> + Sync + 'a;
type Result<T> = std::result::Result<T, CoreError>;
type Bindings = BTreeMap<String, s::BindingValue>;
// Host budgets may raise iterative expansion depth. Recursive source walks
// retain a separate native-stack ceiling, including across import boundaries.
const NATIVE_RECURSION_LIMIT: usize = 128;

/// Compile one trusted source bundle without filesystem access or tool execution.
pub fn compile(
    source: &str,
    resolver: Option<&SourceResolver<'_>>,
    origin: &str,
    limits: ExecutionLimits,
) -> Result<s::Program> {
    limits
        .validate()
        .map_err(|e| structure("resource_limit", e, SourceSpan::default()))?;
    std::thread::scope(|scope| {
        let worker = std::thread::Builder::new()
            .name("eal-composition".into())
            .stack_size(16 * 1024 * 1024)
            .spawn_scoped(scope, || {
                let mut lowerer = Lowerer::new(limits, resolver);
                let syntax = lowerer.read_source(source)?;
                lowerer.import_stack.push(origin.to_owned());
                lowerer.preload_imports(&syntax.declarations, origin)?;
                lowerer.block(&syntax.declarations, &Scope::root(origin))?;
                lowerer.import_stack.pop();
                let mut program = lowerer.finish()?;
                let mut digest = format!("{:x}", Sha256::digest(source.as_bytes()));
                if !program.imports.is_empty() {
                    let bundle = serde_json::json!({"source":digest,"imports":program.imports});
                    digest = format!(
                        "{:x}",
                        Sha256::digest(ascii_bundle(&bundle, &program.limits)?)
                    );
                }
                program.language = syntax.language.clone();
                program.profile = syntax.profile.clone();
                program.source_digest = digest;
                program.authored = Some(syntax);
                program.root_origin = origin.into();
                Ok(program)
            })
            .map_err(|error| {
                structure(
                    "resource_limit",
                    format!("Cannot start composition worker: {error}"),
                    SourceSpan::default(),
                )
            })?;
        worker.join().map_err(|_| {
            structure(
                "composition_failure",
                "Composition worker failed",
                SourceSpan::default(),
            )
        })?
    })
}

/// Rebuild retained authored declarations using only the captured import sources.
pub fn relower(program: &s::Program) -> Result<s::Program> {
    program
        .limits
        .validate()
        .map_err(|error| structure("resource_limit", error, SourceSpan::default()))?;
    crate::validation::structural_value_count(program, program.limits.structural_values)
        .map_err(|error| structure("resource_limit", error, SourceSpan::default()))?;
    if let Some(authored) = &program.authored {
        std::thread::scope(|scope| {
            std::thread::Builder::new()
                .name("eal-recomposition".into())
                .stack_size(16 * 1024 * 1024)
                .spawn_scoped(scope, || {
                    let mut lowerer = Lowerer::new(program.limits.clone(), None);
                    lowerer.sources = program.import_sources.clone();
                    lowerer.import_stack.push(program.root_origin.clone());
                    lowerer.preload_imports(&authored.declarations, &program.root_origin)?;
                    lowerer.block(&authored.declarations, &Scope::root(&program.root_origin))?;
                    let mut result = lowerer.finish()?;
                    result.authored = Some(authored.clone());
                    result.root_origin = program.root_origin.clone();
                    result.language = program.language.clone();
                    result.profile = authored.profile.clone();
                    result.source_digest = program.source_digest.clone();
                    Ok(result)
                })
                .map_err(|error| {
                    structure("resource_limit", error.to_string(), SourceSpan::default())
                })?
                .join()
                .map_err(|_| {
                    structure(
                        "composition_failure",
                        "Recomposition worker failed",
                        SourceSpan::default(),
                    )
                })?
        })
    } else {
        direct::lower_direct(program)
    }
}

pub(crate) fn structure(
    code: &'static str,
    message: impl Into<String>,
    span: SourceSpan,
) -> CoreError {
    CoreError::Structure {
        code,
        message: message.into(),
        span,
    }
}
fn qualify(prefix: &str, name: &str) -> String {
    if prefix.is_empty() {
        name.into()
    } else {
        format!("{prefix}.{name}")
    }
}
fn candidates(prefix: &str, name: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut current = prefix;
    while !current.is_empty() {
        result.push(qualify(current, name));
        current = current.rsplit_once('.').map_or("", |(p, _)| p);
    }
    result.push(name.into());
    result
}
fn operation(reference: &str) -> Option<(&str, &str)> {
    for name in ["head", "tail"] {
        if let Some(inner) = reference
            .strip_prefix(name)
            .and_then(|r| r.strip_prefix('('))
            .and_then(|r| r.strip_suffix(')'))
            && !inner.contains(['(', ')'])
        {
            return Some((name, inner));
        }
    }
    None
}

struct Lowerer<'a> {
    limits: ExecutionLimits,
    resolver: Option<&'a SourceResolver<'a>>,
    sources: BTreeMap<String, String>,
    parsed_sources: BTreeMap<String, crate::Program>,
    imports: BTreeMap<String, String>,
    import_stack: Vec<String>,
    bytes: usize,
    tokens: usize,
    records: BTreeMap<&'static str, IndexMap<String, Record>>,
    symbols: BTreeMap<String, String>,
    directives: Vec<Record>,
    namespaces: BTreeSet<String>,
    locations: BTreeMap<String, SourceSpan>,
    source_files: BTreeMap<String, String>,
    generated: Vec<String>,
    duplicates: Vec<String>,
    errors: Vec<s::Diagnostic>,
    expanded: usize,
    references: usize,
    checked: BTreeSet<String>,
    calls: Vec<Call>,
}
impl<'a> Lowerer<'a> {
    fn new(limits: ExecutionLimits, resolver: Option<&'a SourceResolver<'a>>) -> Self {
        let records = [
            "environment",
            "tool",
            "evidence",
            "assumption",
            "reasoning",
            "claim",
            "argument",
            "objection",
            "pattern",
            "application",
        ]
        .into_iter()
        .map(|k| (k, IndexMap::new()))
        .collect();
        Self {
            limits,
            resolver,
            sources: BTreeMap::new(),
            parsed_sources: BTreeMap::new(),
            imports: BTreeMap::new(),
            import_stack: Vec::new(),
            bytes: 0,
            tokens: 0,
            records,
            symbols: BTreeMap::new(),
            directives: Vec::new(),
            namespaces: BTreeSet::new(),
            locations: BTreeMap::new(),
            source_files: BTreeMap::new(),
            generated: Vec::new(),
            duplicates: Vec::new(),
            errors: Vec::new(),
            expanded: 0,
            references: 0,
            checked: BTreeSet::new(),
            calls: Vec::new(),
        }
    }
    fn read_source(&mut self, source: &str) -> Result<crate::Program> {
        self.bytes = self.bytes.saturating_add(source.len());
        if self.bytes > self.limits.source_bytes {
            return Err(structure(
                "resource_limit",
                "Source bundle byte budget exceeded",
                SourceSpan::default(),
            ));
        }
        let syntax = eal_parser::parse_with_limits(
            source,
            eal_parser::ParseLimits {
                max_source_bytes: self.limits.source_bytes,
                max_tokens: self.limits.source_tokens,
                max_nesting: self.limits.expansion_depth,
                ..Default::default()
            },
        )
        .map_err(CoreError::Recognition)?;
        self.tokens = self.tokens.saturating_add(syntax.tokens.len());
        if self.tokens > self.limits.source_tokens {
            return Err(structure(
                "resource_limit",
                "Source bundle token budget exceeded",
                SourceSpan::default(),
            ));
        }
        crate::lower(&syntax)
    }
    fn error(
        &mut self,
        code: &str,
        message: impl Into<String>,
        owner: Option<&str>,
        span: Option<SourceSpan>,
        expected: Option<&str>,
        actual: Option<&str>,
    ) {
        self.errors.push(s::Diagnostic {
            code: code.into(),
            message: message.into(),
            declaration: owner.map(str::to_owned),
            span: span.or_else(|| owner.and_then(|n| self.locations.get(n).copied())),
            expected: expected.map(str::to_owned),
            actual: actual.map(str::to_owned),
        });
    }
    fn register(&mut self, mut value: Entity, scope: &Scope) {
        if self.symbols.len() + self.directives.len() >= self.limits.declarations {
            self.error(
                "resource_limit",
                "Declaration budget exceeded",
                scope.application.as_deref().or(Some(value.name())),
                Some(scope.span),
                None,
                None,
            );
            return;
        }
        if matches!(value, Entity::Directive(_)) {
            self.directives.push(Record {
                value,
                scope: scope.clone(),
            });
            return;
        }
        let name = qualify(&scope.prefix, value.name());
        let kind = value.kind();
        let compact = matches!(&value,Entity::Argument(v) if v.origin.is_some())
            && self.records["application"].contains_key(&name);
        if (self.symbols.contains_key(&name) && !compact) || self.namespaces.contains(&name) {
            self.duplicates.push(name.clone());
        }
        self.symbols.insert(name.clone(), kind.into());
        value.rename(name.clone());
        self.records.get_mut(kind).expect("entity table").insert(
            name.clone(),
            Record {
                value: value.clone(),
                scope: scope.clone(),
            },
        );
        self.locations.insert(name.clone(), scope.span);
        if let Some(origin) = &scope.source_file {
            self.source_files.insert(name.clone(), origin.clone());
        }
        if let Some(application) = &scope.application
            && !matches!(value, Entity::Application(_))
        {
            self.generated.push(name);
            for (reference, _, many) in value.references() {
                self.references += match scope.bindings.get(&reference) {
                    Some(s::BindingValue::List(v)) if many => v.len(),
                    _ => 1,
                };
            }
            if self.references > self.limits.expanded_references {
                self.error(
                    "resource_limit",
                    "Expanded reference budget exceeded",
                    Some(application),
                    Some(scope.span),
                    None,
                    None,
                );
            }
        }
    }
    fn block(&mut self, declarations: &[Declaration], scope: &Scope) -> Result<()> {
        self.block_at(declarations, scope, 0)
    }
    fn block_at(
        &mut self,
        declarations: &[Declaration],
        scope: &Scope,
        native_depth: usize,
    ) -> Result<()> {
        if native_depth > NATIVE_RECURSION_LIMIT {
            self.error(
                "resource_limit",
                "Native source composition depth budget exceeded",
                scope.application.as_deref().or(Some(&scope.prefix)),
                Some(scope.span),
                None,
                None,
            );
            return Ok(());
        }
        if scope.depth > self.limits.expansion_depth {
            self.error(
                "resource_limit",
                "Composition depth budget exceeded",
                scope.application.as_deref().or(Some(&scope.prefix)),
                Some(scope.span),
                None,
                None,
            );
            return Ok(());
        }
        for declaration in declarations {
            let mut current = scope.clone();
            current.span = declaration.span;
            match &declaration.value {
                DeclarationKind::Context { defaults, body } => {
                    for field in defaults {
                        current.defaults.insert(field.name().into(), field.clone());
                    }
                    current.depth += 1;
                    self.block_at(body, &current, native_depth + 1)?;
                }
                DeclarationKind::Module { name, body } => {
                    self.namespace(&mut current, name);
                    self.block_at(body, &current, native_depth + 1)?;
                }
                DeclarationKind::Import { path, alias } => {
                    let resolved = self.import_path(
                        current.source_file.as_deref().unwrap_or(""),
                        path,
                        declaration.span,
                    )?;
                    if self.import_stack.contains(&resolved) {
                        return Err(structure(
                            "IMPORT_CYCLE",
                            format!(
                                "Source import cycle at {resolved_repr}",
                                resolved_repr = repr(&resolved)
                            ),
                            declaration.span,
                        ));
                    }
                    if self.import_stack.len().saturating_sub(1)
                        >= self.limits.expansion_depth.min(NATIVE_RECURSION_LIMIT)
                    {
                        return Err(structure(
                            "resource_limit",
                            "Source import depth budget exceeded",
                            declaration.span,
                        ));
                    }
                    let imported =
                        self.parsed_sources.get(&resolved).cloned().ok_or_else(|| {
                            structure(
                                "IMPORT_RESOLVER",
                                "Imported source was not captured before composition",
                                declaration.span,
                            )
                        })?;
                    self.import_stack.push(resolved.clone());
                    current.defaults.clear();
                    current.source_file = Some(resolved);
                    self.namespace(&mut current, alias);
                    self.block_at(&imported.declarations, &current, native_depth + 1)?;
                    self.import_stack.pop();
                }

                DeclarationKind::Argument {
                    name,
                    body: Some(body),
                    flow,
                } => {
                    let parent = current.prefix.clone();
                    current.prefix = qualify(&parent, name);
                    current.depth += 1;
                    self.block_at(body, &current, native_depth + 1)?;
                    let mut value = values::argument(name, flow);
                    if let (Some(p), Some(a)) = (&current.pattern, &current.application) {
                        value.origin = Some(s::ArgumentOrigin {
                            pattern: p.clone(),
                            application: a.clone(),
                        });
                    }
                    let mut registration = current.clone();
                    registration.prefix = parent;
                    registration.depth -= 1;
                    self.register(Entity::Argument(value), &registration);
                    let full = qualify(&registration.prefix, name);
                    if let Some(record) = self
                        .records
                        .get_mut("argument")
                        .and_then(|r| r.get_mut(&full))
                    {
                        record.scope.prefix = current.prefix;
                    }
                }
                DeclarationKind::Guard {
                    parameter,
                    body,
                    otherwise,
                } => {
                    if current.pattern.is_none() {
                        self.error(
                            "invalid_pattern_guard",
                            "when is a structural list guard inside a pattern",
                            Some(&current.prefix),
                            Some(current.span),
                            None,
                            None,
                        );
                        continue;
                    }
                    let Some(s::BindingValue::List(items)) = current.bindings.get(parameter) else {
                        self.error(
                            "pattern_guard_type",
                            "when requires a typed list parameter",
                            current.application.as_deref(),
                            Some(current.span),
                            None,
                            None,
                        );
                        continue;
                    };
                    if !items.is_empty() {
                        current.guards.insert(parameter.clone());
                        current.depth += 1;
                        self.block_at(body, &current, native_depth + 1)?;
                    } else if let Some(otherwise) = otherwise {
                        current.depth += 1;
                        self.block_at(otherwise, &current, native_depth + 1)?;
                    }
                }
                _ => {
                    let mut value = values::entity(declaration, &current.defaults)?;
                    if let Entity::Argument(argument) = &mut value
                        && let (Some(p), Some(a)) = (&current.pattern, &current.application)
                    {
                        argument.origin = Some(s::ArgumentOrigin {
                            pattern: p.clone(),
                            application: a.clone(),
                        });
                    }
                    if let Entity::Directive(directive) = &mut value {
                        directive.source_file = current.source_file.clone();
                    }
                    self.register(value, &current);
                }
            }
        }
        Ok(())
    }
    fn namespace(&mut self, scope: &mut Scope, name: &str) {
        let namespace = qualify(&scope.prefix, name);
        if self.namespaces.contains(&namespace) || self.symbols.contains_key(&namespace) {
            self.error(
                "duplicate_symbol",
                format!(
                    "Namespace {namespace_repr} is declared more than once",
                    namespace_repr = repr(&namespace)
                ),
                Some(&namespace),
                Some(scope.span),
                None,
                None,
            );
        }
        self.namespaces.insert(namespace.clone());
        scope.prefix = namespace;
        scope.depth += 1;
    }
    fn import_path(&self, origin: &str, path: &str, span: SourceSpan) -> Result<String> {
        if path.is_empty() || path.contains('\\') || path.starts_with('/') {
            return Err(structure(
                "IMPORT_PATH",
                "Imports require a relative POSIX source path",
                span,
            ));
        }
        let mut parts: Vec<&str> = origin.rsplit_once('/').map_or(Vec::new(), |(parent, _)| {
            parent
                .split('/')
                .filter(|s| !s.is_empty() && *s != ".")
                .collect()
        });
        for part in path.split('/') {
            match part {
                "" | "." => {}
                ".." => {
                    if parts.pop().is_none() {
                        return Err(structure(
                            "IMPORT_PATH",
                            "Import escapes the source root",
                            span,
                        ));
                    }
                }
                _ => parts.push(part),
            }
        }
        Ok(parts.join("/"))
    }
    fn finish(mut self) -> Result<s::Program> {
        let patterns: Vec<_> = self.records["pattern"].values().cloned().collect();
        for record in patterns {
            self.check_template(&record)?;
        }
        self.check_recursive_components();
        let mut done = BTreeSet::new();
        if self.records["application"].len() > self.limits.applications {
            self.error(
                "resource_limit",
                "Pattern application budget exceeded",
                None,
                None,
                None,
                None,
            );
            done.extend(self.records["application"].keys().cloned());
        }
        loop {
            let pending: Vec<_> = self.records["application"]
                .iter()
                .filter(|(n, _)| !done.contains(*n))
                .map(|(_, r)| r.clone())
                .collect();
            if pending.is_empty() {
                break;
            }
            for record in pending {
                done.insert(record.value.name().to_owned());
                self.expand(&record)?;
            }
        }
        if self.errors.iter().any(|e| e.code == "resource_limit") {
            for table in self.records.values_mut() {
                table.retain(|_, r| r.scope.application.is_none());
            }
            self.generated.clear();
        }
        let mut program = s::Program::empty("EAL/3".into(), String::new(), self.limits.clone());
        let records: Vec<_> = self
            .records
            .values()
            .flat_map(|t| t.values().cloned())
            .collect();
        for record in records {
            let value = self.resolved(&record);
            values::insert(&mut program, value);
        }
        let directives = self.directives.clone();
        for record in directives {
            if let Entity::Directive(value) = self.resolved(&record) {
                program.argumentation_directives.push(value);
            }
        }
        let definitions: usize = self.records["pattern"]
            .values()
            .map(|record| {
                let Entity::Pattern(pattern) = &record.value else {
                    unreachable!("pattern table");
                };
                pattern.body.as_ref().map_or(Ok(1), |body| {
                    self.template_imports(body, record.scope.source_file.as_deref().unwrap_or(""))
                        .map(|body| body_count(&body))
                })
            })
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .sum();
        program.declaration_count = self.records.values().map(IndexMap::len).sum::<usize>()
            + definitions
            + program.argumentation_directives.len();
        program.duplicates = self.duplicates;
        program.locations = self.locations;
        program.source_files = self.source_files;
        program.lowering_diagnostics = self.errors;
        program.generated = self.generated;
        program.imports = self.imports;
        program.import_sources = self.sources;
        Ok(program)
    }
}

fn body_count(declarations: &[Declaration]) -> usize {
    declarations
        .iter()
        .map(|d| match &d.value {
            DeclarationKind::Context { body, .. } => body_count(body),
            DeclarationKind::Module { body, .. }
            | DeclarationKind::Argument {
                body: Some(body), ..
            }
            | DeclarationKind::Pattern {
                body: crate::PatternBody::Block(body),
                ..
            } => 1 + body_count(body),
            DeclarationKind::Guard {
                body, otherwise, ..
            } => 1 + body_count(body) + otherwise.as_ref().map_or(0, |b| body_count(b)),
            _ => 1,
        })
        .sum()
}
fn entities(program: &s::Program) -> Vec<Entity> {
    let mut values = Vec::new();
    values.extend(
        program
            .environments
            .values()
            .cloned()
            .map(Entity::Environment),
    );
    values.extend(program.tools.values().cloned().map(Entity::Tool));
    values.extend(program.evidence.values().cloned().map(Entity::Evidence));
    values.extend(
        program
            .assumptions
            .values()
            .cloned()
            .map(Entity::Assumption),
    );
    values.extend(program.reasoning.values().cloned().map(Entity::Reasoning));
    values.extend(program.claims.values().cloned().map(Entity::Claim));
    values.extend(
        program
            .arguments
            .values()
            .filter(|a| a.origin.is_none())
            .cloned()
            .map(Entity::Argument),
    );
    values.extend(program.objections.values().cloned().map(Entity::Objection));
    values.extend(program.patterns.values().cloned().map(Entity::Pattern));
    values.extend(
        program
            .applications
            .values()
            .cloned()
            .map(Entity::Application),
    );
    values.extend(
        program
            .argumentation_directives
            .iter()
            .cloned()
            .map(Entity::Directive),
    );
    values
}

fn ascii_bundle(value: &serde_json::Value, limits: &ExecutionLimits) -> Result<Vec<u8>> {
    let bytes = crate::canonical::canonical_json_bytes(value, limits)
        .map_err(|e| structure("resource_limit", e, SourceSpan::default()))?;
    let source = String::from_utf8(bytes).expect("canonical JSON UTF-8");
    let mut out = String::new();
    for c in source.chars() {
        if u32::from(c) >= 0x7f {
            let mut buffer = [0u16; 2];
            for unit in c.encode_utf16(&mut buffer) {
                use std::fmt::Write;
                write!(out, "\\u{unit:04x}").expect("String write");
            }
        } else {
            out.push(c);
        }
    }
    Ok(out.into_bytes())
}

fn repr(name: &str) -> String {
    crate::expressions::python_repr(&serde_json::Value::String(name.into()))
}
