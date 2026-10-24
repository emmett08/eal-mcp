//! One collection's captured Tempo credentials and immutable response inspection.

use crate::config::TempoConfig;
use eal_collector_contracts::{SensitiveJsonLimits, contains_sensitive_json};
use serde_json::Value;

/// Secret values stay private and are never part of an observation record.
pub(crate) struct Credentials {
    token: Option<String>,
    tenant: Option<String>,
}

impl Credentials {
    pub(super) fn load(selected: &TempoConfig) -> Result<Self, String> {
        Ok(Self {
            token: credential(selected.token_env.as_deref())?,
            tenant: credential(selected.tenant_env.as_deref())?,
        })
    }

    pub(super) fn token(&self) -> Option<&str> {
        self.token.as_deref()
    }

    pub(super) fn tenant(&self) -> Option<&str> {
        self.tenant.as_deref()
    }

    pub(crate) fn inspect(&self, value: &Value) -> Result<(), String> {
        let secrets = [self.token().unwrap_or(""), self.tenant().unwrap_or("")];
        if contains_sensitive_json(value, &secrets, SensitiveJsonLimits::default())
            .map_err(|_| "Tempo response failed credential inspection")?
        {
            return Err("Tempo response contains configured credential material".into());
        }
        Ok(())
    }
}

fn credential(name: Option<&str>) -> Result<Option<String>, String> {
    match name {
        None => Ok(None),
        Some(name) => {
            let value =
                std::env::var(name).map_err(|_| "Configured Tempo credential is unavailable")?;
            if value.is_empty() || value.len() > 8192 || value.chars().any(char::is_control) {
                return Err("Configured Tempo credential is malformed".into());
            }
            Ok(Some(value))
        }
    }
}
