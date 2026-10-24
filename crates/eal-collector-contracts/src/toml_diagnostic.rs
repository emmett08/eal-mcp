//! Safe diagnostics for operator-owned TOML before sensitive fields are decoded.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TomlDocumentKind {
    Registry,
    ServerConfiguration,
    Limits,
}
impl TomlDocumentKind {
    fn code(self) -> &'static str {
        match self {
            Self::Registry => "EAL_REGISTRY_SYNTAX",
            Self::ServerConfiguration => "EAL_SERVER_CONFIGURATION_TOML",
            Self::Limits => "EAL_LIMITS_TOML",
        }
    }
}
/// Report only an application-owned code and numeric location. Parser messages,
/// raw tokens and source excerpts remain private even for malformed input.
pub fn toml_diagnostic(kind: TomlDocumentKind, source: &str, error: &toml::de::Error) -> String {
    let offset = error.span().map_or(0, |span| span.start.min(source.len()));
    let before = &source.as_bytes()[..offset];
    let line = before.iter().filter(|&&byte| byte == b'\n').count() + 1;
    let column = before
        .iter()
        .rposition(|&byte| byte == b'\n')
        .map_or(offset + 1, |index| offset - index);
    format!("{} at line {line}, byte column {column}", kind.code())
}
