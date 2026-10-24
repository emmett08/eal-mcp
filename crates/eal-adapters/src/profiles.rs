//! The sole input, environment and allowance definitions for compatibility adapters.
use crate::Error;
use serde_json::{Value, json};
use std::collections::BTreeMap;

pub const HTTP_TOKEN_ENV: &str = "ENGARG_COLLECTOR_TOKEN";
pub const HTTP_ORIGIN_ENV: &str = "ENGARG_COLLECTOR_ORIGIN";

pub const RESPONSE_MAX_BYTES: usize = 1_048_576;
pub const TRANSPORT_TIMEOUT_SECONDS: u64 = 60;

#[derive(Clone, Copy, Debug)]
pub struct CompatibilityProfile {
    pub id: &'static str,
    pub purpose: &'static str,
    pub arguments: &'static [&'static str],
    pub inherit_env: &'static [&'static str],
    pub fixed_env: &'static [(&'static str, &'static str)],
    pub version: &'static str,
    pub timeout_seconds: u64,
    pub max_output_bytes: usize,
    pub parallel_safe: bool,
    pub acquisition_predicate: &'static str,
}

impl CompatibilityProfile {
    pub fn validate_arguments(self, arguments: &BTreeMap<String, String>) -> Result<(), Error> {
        if arguments.len() != self.arguments.len()
            || self
                .arguments
                .iter()
                .any(|name| !arguments.contains_key(*name))
        {
            return Err(Error::InvalidInput(
                "collector arguments differ from the selected tool contract".into(),
            ));
        }
        Ok(())
    }
    /// Catalogue prompts disclose input names and purpose, without execution settings.
    pub fn planning_view(self) -> Value {
        json!({"id":self.id,"purpose":self.purpose,"arguments":self.arguments})
    }
    /// Build the exact child environment from this operator-selected compatibility profile.
    pub fn environment(self) -> BTreeMap<String, String> {
        self.inherit_env
            .iter()
            .filter_map(|name| {
                std::env::var(name)
                    .ok()
                    .map(|value| ((*name).to_owned(), value))
            })
            .chain(
                self.fixed_env
                    .iter()
                    .map(|(name, value)| ((*name).to_owned(), (*value).to_owned())),
            )
            .collect()
    }
}

const fn profile(
    id: &'static str,
    purpose: &'static str,
    arguments: &'static [&'static str],
    inherit_env: &'static [&'static str],
    fixed_env: &'static [(&'static str, &'static str)],
) -> CompatibilityProfile {
    CompatibilityProfile {
        id,
        purpose,
        arguments,
        inherit_env,
        fixed_env,
        version: "1",
        timeout_seconds: 65,
        max_output_bytes: 4_194_304,
        parallel_safe: false,
        acquisition_predicate: "collected == true",
    }
}

pub const ARTIFACT: CompatibilityProfile = profile(
    "artifact_json",
    "Read explicitly supplied, digest-checked retained JSON artifacts for the exact evidence need. Missing artifacts fail collection; existence does not establish acceptance.",
    &["inputs"],
    &[],
    &[],
);
pub const SCOPED: CompatibilityProfile = profile(
    "scoped_json",
    "Read Kubernetes Application JSON or exact Grafana Tempo search/full traces using an operator-owned inputs file. Unresolved selectors fail before I/O; scope and interpretation require independent review.",
    &["inputs"],
    &["PATH", "HOME", "KUBECONFIG", "TEMPO_TOKEN", "TEMPO_TENANT"],
    &[],
);
pub const KUBECTL: CompatibilityProfile = profile(
    "kubectl_get",
    "Read Kubernetes resource JSON",
    &["context", "namespace", "resource", "name"],
    &["PATH", "HOME", "KUBECONFIG"],
    &[],
);
pub const GITHUB: CompatibilityProfile = profile(
    "gh_api_get",
    "Read a GitHub REST API JSON endpoint",
    &["endpoint"],
    &["PATH", "HOME", "GH_TOKEN", "GH_HOST"],
    &[("GH_PROMPT_DISABLED", "1"), ("GH_PAGER", "cat")],
);
pub const HTTP: CompatibilityProfile = profile(
    "http_json_get",
    "Read an HTTP JSON endpoint",
    &["url"],
    &["PATH", HTTP_TOKEN_ENV, HTTP_ORIGIN_ENV],
    &[],
);
pub const ALL: &[CompatibilityProfile] = &[ARTIFACT, SCOPED, KUBECTL, GITHUB, HTTP];
pub fn get(id: &str) -> Option<&'static CompatibilityProfile> {
    ALL.iter().find(|profile| profile.id == id)
}
pub fn require(id: &str) -> Result<&'static CompatibilityProfile, Error> {
    get(id).ok_or_else(|| {
        Error::InvalidInput("collector needs an operator-selected descriptor".into())
    })
}
