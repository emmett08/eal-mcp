//! Literal argv templates and typed command-registry serialisation.
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CommandTemplate {
    pub argv: Vec<String>,
    pub inherit_env: Vec<String>,
    pub timeout_seconds: u64,
    pub max_output_bytes: usize,
    pub parallel_safe: bool,
}
impl CommandTemplate {
    pub(crate) fn validate(&self, parameters: &[String]) -> Result<(), String> {
        if self.argv.is_empty()
            || self.argv.len() > 256
            || !(1..=3600).contains(&self.timeout_seconds)
            || !(1..=16_777_216).contains(&self.max_output_bytes)
            || self.inherit_env.len() > 256
            || self.inherit_env.iter().any(|v| !crate::environment_name(v))
            || self
                .inherit_env
                .iter()
                .collect::<std::collections::BTreeSet<_>>()
                .len()
                != self.inherit_env.len()
        {
            return Err("Invalid command template allowances or environment names".into());
        }
        for (index, value) in self.argv.iter().enumerate() {
            match placeholder(value)? {
                Some(Placeholder::Parameter(name))
                    if index == 0 || !parameters.iter().any(|p| p == name) =>
                {
                    return Err("Command template uses an unknown or executable parameter".into());
                }
                _ => {}
            }
        }
        Ok(())
    }
    pub(crate) fn bind(
        &self,
        version: &str,
        parameters: &BTreeMap<String, String>,
        operators: &BTreeMap<String, String>,
    ) -> Result<CommandBinding, String> {
        let argv = self
            .argv
            .iter()
            .map(|value| match placeholder(value)? {
                None => Ok(value.clone()),
                Some(Placeholder::Parameter(name)) => parameters
                    .get(name)
                    .cloned()
                    .ok_or_else(|| format!("Missing collector parameter {name}")),
                Some(Placeholder::Operator(name)) => operators
                    .get(name)
                    .cloned()
                    .ok_or_else(|| format!("Missing operator selection {name}")),
            })
            .collect::<Result<Vec<_>, _>>()?;
        if argv
            .iter()
            .any(|v| v.is_empty() || v.len() > 8192 || v.chars().any(char::is_control))
        {
            return Err("Bound command arguments must be bounded literals without controls".into());
        }
        Ok(CommandBinding {
            kind: CommandKind::Command,
            version: version.into(),
            argv,
            timeout_seconds: self.timeout_seconds,
            max_output_bytes: self.max_output_bytes,
            inherit_env: self.inherit_env.clone(),
            parallel_safe: self.parallel_safe,
        })
    }
}

enum Placeholder<'a> {
    Parameter(&'a str),
    Operator(&'a str),
}
fn placeholder(value: &str) -> Result<Option<Placeholder<'_>>, String> {
    if value.is_empty() || value.len() > 8192 || value.chars().any(char::is_control) {
        return Err("Command arguments must be bounded literals without controls".into());
    }
    if !value.contains(['{', '}']) {
        return Ok(None);
    }
    let name = value
        .strip_prefix('{')
        .and_then(|value| value.strip_suffix('}'))
        .ok_or("Template placeholders must occupy an entire argv token")?;
    if let Some(name) = name.strip_prefix("operator.") {
        if crate::identifier(name) {
            return Ok(Some(Placeholder::Operator(name)));
        }
    } else if crate::identifier(name) {
        return Ok(Some(Placeholder::Parameter(name)));
    }
    Err("Invalid command template placeholder".into())
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CommandBinding {
    kind: CommandKind,
    pub version: String,
    pub argv: Vec<String>,
    pub timeout_seconds: u64,
    pub max_output_bytes: usize,
    pub inherit_env: Vec<String>,
    pub parallel_safe: bool,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
enum CommandKind {
    Command,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CommandRegistry {
    pub tools: BTreeMap<String, CommandBinding>,
}
impl CommandRegistry {
    pub fn to_toml(&self) -> Result<String, String> {
        toml::to_string_pretty(self).map_err(|error| error.to_string())
    }
}
