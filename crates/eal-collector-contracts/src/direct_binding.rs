//! Concrete transport and response mappings; these confer no inferential authority.
use crate::{PinnedFile, ToolKind};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResponseFormat {
    #[default]
    Structured,
    JsonText,
}

/// Identity declared by an MCP server at initialization. It identifies the
/// endpoint's advertised implementation, not independently verified source code.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct McpServerIdentity {
    pub name: String,
    pub version: String,
    pub protocol: String,
}
impl McpServerIdentity {
    pub fn validate(&self) -> Result<(), String> {
        if [&self.name, &self.version, &self.protocol]
            .iter()
            .any(|s| s.is_empty() || s.len() > 1024 || s.chars().any(char::is_control))
        {
            return Err("MCP server identity requires bounded name, version and protocol".into());
        }
        Ok(())
    }
}

/// Direct transport settings live beside the operation rather than in a catalogue.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DirectBinding {
    pub tool: Option<String>,
    pub url: Option<String>,
    pub token_env: Option<String>,
    #[serde(default)]
    pub headers_env: BTreeMap<String, String>,
    #[serde(default)]
    pub response_format: ResponseFormat,
    #[serde(default)]
    pub text_block: usize,
    #[serde(default)]
    pub value_pointer: String,
    /// An omitted pointer requests collection time for a new measurement.
    pub observed_at_pointer: Option<String>,
    pub input_schema_sha256: Option<String>,
    pub output_schema_sha256: Option<String>,
    /// Preparation pins the advertised implementation as well as its schemas.
    pub server_identity: Option<McpServerIdentity>,
    /// Optional pinned JSON Schemas for arguments and normalised values.
    pub input_schema: Option<PinnedFile>,
    pub output_schema: Option<PinnedFile>,
}
impl DirectBinding {
    pub fn validate(&self, kind: ToolKind) -> Result<(), String> {
        let mcp = matches!(kind, ToolKind::McpStdio | ToolKind::McpHttp);
        let network = matches!(kind, ToolKind::McpHttp | ToolKind::HttpGet);
        if mcp != self.tool.is_some()
            || self
                .tool
                .as_deref()
                .is_some_and(|s| s.is_empty() || s.len() > 256 || s.chars().any(char::is_control))
            || network != self.url.is_some()
            || (!network && (self.token_env.is_some() || !self.headers_env.is_empty()))
            || (kind != ToolKind::HttpGet && !self.headers_env.is_empty())
            || self
                .token_env
                .as_deref()
                .is_some_and(|s| !crate::environment_name(s))
            || self.headers_env.len() > 16
            || self
                .headers_env
                .values()
                .any(|s| !crate::environment_name(s))
            || !pointer(&self.value_pointer)
            || self
                .observed_at_pointer
                .as_deref()
                .is_some_and(|p| !pointer(p))
            || self.text_block > 127
            || (self.response_format == ResponseFormat::Structured && self.text_block != 0)
            || (!mcp
                && (self.response_format != ResponseFormat::Structured
                    || self.input_schema_sha256.is_some()
                    || self.output_schema_sha256.is_some()
                    || self.server_identity.is_some()))
            || (self.output_schema_sha256.is_some() && self.input_schema_sha256.is_none())
            || self
                .server_identity
                .as_ref()
                .is_some_and(|identity| identity.validate().is_err())
            || self
                .input_schema_sha256
                .iter()
                .chain(self.output_schema_sha256.iter())
                .any(|s| !digest(s))
            || self
                .input_schema
                .iter()
                .chain(self.output_schema.iter())
                .any(|p| {
                    p.path.is_empty()
                        || p.path.len() > 8192
                        || p.path.chars().any(char::is_control)
                        || !digest(&p.sha256)
                })
        {
            return Err(
                "EAL_DIRECT_BINDING: inconsistent transport, response mapping or schema pins"
                    .into(),
            );
        }
        if let Some(url) = &self.url {
            if url.len() > 8192
                || url.chars().any(char::is_whitespace)
                || url.chars().any(char::is_control)
            {
                return Err("EAL_DIRECT_URL: invalid bounded HTTP URL".into());
            }
            let parsed = url::Url::parse(url).map_err(|_| "EAL_DIRECT_URL: malformed URL")?;
            let loopback = parsed.host_str().is_some_and(|h| {
                h == "localhost"
                    || h.trim_matches(['[', ']'])
                        .parse::<std::net::IpAddr>()
                        .is_ok_and(|ip| ip.is_loopback())
            });
            if !(parsed.scheme() == "https" || (parsed.scheme() == "http" && loopback))
                || parsed.host_str().is_none()
                || parsed.port() == Some(0)
                || !parsed.username().is_empty()
                || parsed.password().is_some()
                || parsed.query().is_some()
                || parsed.fragment().is_some()
            {
                return Err("EAL_DIRECT_URL: HTTPS or loopback HTTP without credentials, query or fragment is required".into());
            }
        }
        Ok(())
    }
}
fn pointer(s: &str) -> bool {
    if s.len() > 4096 || (!s.is_empty() && !s.starts_with('/')) || s.chars().any(char::is_control) {
        return false;
    }
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '~' && !matches!(chars.next(), Some('0' | '1')) {
            return false;
        }
    }
    true
}
fn digest(s: &str) -> bool {
    s.len() == 64
        && s.bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
