//! Combine optional native and MCP discovery without acquiring observations.
use super::NativeSources;
use crate::{
    BridgeError, BridgeErrorKind, EvidenceBridge, EvidenceConfig,
    discovery::{self, DiscoveryGap, DiscoveryReport},
};
use eal_collector_contracts::capability::{CAPABILITY_CATALOGUE_SCHEMA, CapabilityCatalogue};
use std::{collections::BTreeMap, time::Instant};

#[derive(Clone)]
pub struct EvidenceSources {
    bridge: EvidenceBridge,
    native: NativeSources,
}
impl EvidenceSources {
    pub fn from_config(
        config: EvidenceConfig,
        environment: BTreeMap<String, String>,
    ) -> Result<Self, String> {
        config.validate()?;
        let native = NativeSources::new(config.native_providers.clone(), environment.clone())?;
        let bridge = EvidenceBridge::new(config, environment)?;
        Ok(Self { bridge, native })
    }
    pub fn bridge(&self) -> &EvidenceBridge {
        &self.bridge
    }
    pub fn native(&self) -> &NativeSources {
        &self.native
    }
    pub fn discover(&self) -> Result<DiscoveryReport, BridgeError> {
        let started = Instant::now();
        let mut report = self.bridge.discover()?;
        let native_catalogue = CapabilityCatalogue {
            schema: CAPABILITY_CATALOGUE_SCHEMA.into(),
            capabilities: self
                .native
                .configurations()
                .map(|configuration| configuration.capability.clone())
                .collect(),
        };
        report.catalogue.capabilities =
            discovery::combine([report.catalogue.clone(), native_catalogue]).map_err(|_| {
                BridgeError::new(
                    BridgeErrorKind::Configuration,
                    "Conflicting native and MCP capability catalogues",
                )
            })?;
        if report.catalogue.capabilities.len() > report.limits.candidates {
            return Err(BridgeError::new(
                BridgeErrorKind::Budget,
                "Combined source catalogue exceeds the configured candidate budget",
            ));
        }
        report.gaps.retain(|gap| {
            !gap.capability_id
                .as_ref()
                .is_some_and(|id| self.native.contains(id))
        });
        for configuration in self.native.configurations() {
            let id = &configuration.capability.id;
            match self.native.admit(id) {
                Ok(()) => report.admitted_capabilities.push(id.clone()),
                Err(error) => report.gaps.push(DiscoveryGap {
                    capability_id: Some(id.clone()),
                    connection_id: None,
                    kind: format!("{:?}", error.kind).to_ascii_lowercase(),
                    message: error.message,
                }),
            }
        }
        report.admitted_capabilities.sort();
        report.admitted_capabilities.dedup();
        report.catalogue.validate().map_err(|_| {
            BridgeError::new(
                BridgeErrorKind::Configuration,
                "Combined source catalogue exceeds capability bounds",
            )
        })?;
        report.metrics.elapsed_ms =
            u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX);
        Ok(report)
    }
}
