//! Capture every authored import before expansion, including unused template branches.
use super::*;
impl Lowerer<'_> {
    pub(super) fn preload_imports(
        &mut self,
        declarations: &[Declaration],
        origin: &str,
    ) -> Result<()> {
        self.preload_imports_at(declarations, origin, 0)
    }
    fn preload_imports_at(
        &mut self,
        declarations: &[Declaration],
        origin: &str,
        native_depth: usize,
    ) -> Result<()> {
        if native_depth > NATIVE_RECURSION_LIMIT {
            return Err(structure(
                "resource_limit",
                "Native source import depth budget exceeded",
                SourceSpan::default(),
            ));
        }
        for declaration in declarations {
            match &declaration.value {
                DeclarationKind::Import { path, .. } => {
                    let resolved = self.import_path(origin, path, declaration.span)?;
                    if self.import_stack.contains(&resolved) {
                        return Err(structure(
                            "IMPORT_CYCLE",
                            format!("Source import cycle at {}", repr(&resolved)),
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
                    let source = if let Some(source) = self.sources.get(&resolved) {
                        source.clone()
                    } else {
                        let resolver = self.resolver.ok_or_else(|| {
                            structure(
                                "IMPORT_RESOLVER",
                                "Source imports require a trusted host source resolver",
                                declaration.span,
                            )
                        })?;
                        let source = resolver(&resolved).map_err(|e| {
                            structure(
                                "IMPORT_RESOLVER",
                                format!("Cannot resolve source import {}: {e}", repr(&resolved)),
                                declaration.span,
                            )
                        })?;
                        self.sources.insert(resolved.clone(), source.clone());
                        source
                    };
                    self.import_stack.push(resolved.clone());
                    let syntax = self.read_source(&source)?;
                    self.preload_imports_at(&syntax.declarations, &resolved, native_depth + 1)?;
                    self.import_stack.pop();
                    self.imports.insert(
                        resolved.clone(),
                        format!("{:x}", Sha256::digest(source.as_bytes())),
                    );
                    self.parsed_sources.insert(resolved, syntax);
                }
                DeclarationKind::Context { body, .. }
                | DeclarationKind::Module { body, .. }
                | DeclarationKind::Argument {
                    body: Some(body), ..
                }
                | DeclarationKind::Pattern {
                    body: crate::PatternBody::Block(body),
                    ..
                } => self.preload_imports_at(body, origin, native_depth + 1)?,
                DeclarationKind::Guard {
                    body, otherwise, ..
                } => {
                    self.preload_imports_at(body, origin, native_depth + 1)?;
                    if let Some(otherwise) = otherwise {
                        self.preload_imports_at(otherwise, origin, native_depth + 1)?;
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }
    /// An imported namespace has a lexical body while the public AST retains its source path.
    pub(super) fn template_imports(
        &self,
        declarations: &[Declaration],
        origin: &str,
    ) -> Result<Vec<Declaration>> {
        declarations
            .iter()
            .map(|declaration| {
                let mut result = declaration.clone();
                match &mut result.value {
                    DeclarationKind::Import { path, alias } => {
                        let resolved = self.import_path(origin, path, declaration.span)?;
                        let imported = self.parsed_sources.get(&resolved).ok_or_else(|| {
                            structure(
                                "IMPORT_RESOLVER",
                                "Imported source was not captured",
                                declaration.span,
                            )
                        })?;
                        let body = self.template_imports(&imported.declarations, &resolved)?;
                        result.value = DeclarationKind::Module {
                            name: alias.clone(),
                            body,
                        };
                    }
                    DeclarationKind::Context { body, .. }
                    | DeclarationKind::Module { body, .. }
                    | DeclarationKind::Argument {
                        body: Some(body), ..
                    }
                    | DeclarationKind::Pattern {
                        body: crate::PatternBody::Block(body),
                        ..
                    } => *body = self.template_imports(body, origin)?,
                    DeclarationKind::Guard {
                        body, otherwise, ..
                    } => {
                        *body = self.template_imports(body, origin)?;
                        if let Some(otherwise) = otherwise {
                            *otherwise = self.template_imports(otherwise, origin)?;
                        }
                    }
                    _ => {}
                }
                Ok(result)
            })
            .collect()
    }
}
