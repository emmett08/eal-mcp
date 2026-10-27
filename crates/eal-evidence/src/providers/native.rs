//! Inspect configured native candidate identities without running their tools.
use super::{NativeProviderConfig, NativeTransport};
use crate::{BridgeError, BridgeErrorKind};
use eal_collector_contracts::capability::canonical_identity;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs::{File, OpenOptions},
    io::Read,
    path::Path,
    sync::Arc,
};

#[derive(Clone)]
pub struct NativeSources {
    providers: Arc<BTreeMap<String, NativeProviderConfig>>,
    environment: Arc<BTreeMap<String, String>>,
}
fn failure(kind: BridgeErrorKind, message: impl Into<String>) -> BridgeError {
    BridgeError {
        kind,
        message: message.into(),
    }
}
fn configuration(message: impl Into<String>) -> BridgeError {
    failure(BridgeErrorKind::Configuration, message)
}
fn open_regular(path: &Path) -> Result<File, BridgeError> {
    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(
            (rustix::fs::OFlags::NOFOLLOW | rustix::fs::OFlags::NONBLOCK).bits() as i32,
        );
    }
    let file = options
        .open(path)
        .map_err(|_| configuration("Native source file is unavailable or is a symbolic link"))?;
    if !file
        .metadata()
        .map_err(|_| configuration("Cannot inspect native source file"))?
        .is_file()
    {
        return Err(configuration("Native source path must be a regular file"));
    }
    Ok(file)
}
fn file_digest(path: &Path, maximum: u64) -> Result<String, BridgeError> {
    let mut file = open_regular(path)?;
    if file
        .metadata()
        .map_err(|_| configuration("Cannot inspect native source file"))?
        .len()
        > maximum
    {
        return Err(configuration(
            "Native source file exceeds fingerprint allowance",
        ));
    }
    let mut hash = Sha256::new();
    let mut block = [0u8; 65536];
    let mut total = 0u64;
    loop {
        let count = file
            .read(&mut block)
            .map_err(|_| configuration("Cannot fingerprint native source file"))?;
        if count == 0 {
            break;
        }
        total = total.saturating_add(count as u64);
        if total > maximum {
            return Err(configuration(
                "Native source file exceeds fingerprint allowance",
            ));
        }
        hash.update(&block[..count]);
    }
    Ok(format!("{:x}", hash.finalize()))
}

impl NativeSources {
    pub fn new(
        configurations: Vec<NativeProviderConfig>,
        environment: BTreeMap<String, String>,
    ) -> Result<Self, String> {
        if configurations.len() > 128 {
            return Err("Too many configured native providers".into());
        }
        let mut providers = BTreeMap::new();
        for configuration in configurations {
            configuration.validate()?;
            if providers
                .insert(configuration.capability.id.clone(), configuration)
                .is_some()
            {
                return Err("Native provider repeats a capability identity".into());
            }
        }
        Ok(Self {
            providers: Arc::new(providers),
            environment: Arc::new(environment),
        })
    }
    pub fn contains(&self, id: &str) -> bool {
        self.providers.contains_key(id)
    }
    pub fn configurations(&self) -> impl Iterator<Item = &NativeProviderConfig> {
        self.providers.values()
    }
    fn selected(&self, id: &str) -> Result<&NativeProviderConfig, BridgeError> {
        self.providers
            .get(id)
            .ok_or_else(|| configuration("Native capability has no configured execution authority"))
    }
    fn environment(
        &self,
        config: &NativeProviderConfig,
    ) -> Result<BTreeMap<String, String>, BridgeError> {
        config
            .environment_names()
            .into_iter()
            .map(|name| {
                let value = self
                    .environment
                    .get(name)
                    .filter(|value| {
                        !value.is_empty() && value.len() <= 65536 && !value.contains('\0')
                    })
                    .ok_or_else(|| {
                        configuration(
                            "Native provider requires an unavailable environment reference",
                        )
                    })?;
                Ok((name.to_owned(), value.clone()))
            })
            .collect()
    }
    /// Validate the configured implementation without acquiring evidence.
    pub fn admit(&self, id: &str) -> Result<(), BridgeError> {
        let config = self.selected(id)?;
        config
            .capability
            .measurement_identity()
            .map_err(configuration)?;
        self.fingerprints(config)?;
        Ok(())
    }
    fn fingerprints(&self, config: &NativeProviderConfig) -> Result<(String, String), BridgeError> {
        let mut files = Vec::new();
        if let NativeTransport::Command {
            program,
            supervisor,
            ..
        } = &config.transport
        {
            files.push((program.clone(), file_digest(program, 256 * 1024 * 1024)?));
            files.push((
                supervisor.clone(),
                file_digest(supervisor, 256 * 1024 * 1024)?,
            ));
        }
        if let NativeTransport::RetainedArtifact { path, sha256 } = &config.transport {
            let actual = file_digest(path, config.capability.limits.output_bytes)?;
            if &actual != sha256 {
                return Err(failure(
                    BridgeErrorKind::SchemaDrift,
                    "Retained artifact content differs from its approved hash",
                ));
            }
            files.push((path.clone(), actual));
        }
        for pin in &config.pinned_files {
            let actual = file_digest(&pin.path, 256 * 1024 * 1024)?;
            if actual != pin.sha256 {
                return Err(failure(
                    BridgeErrorKind::SchemaDrift,
                    "Native source package/configuration file changed after approval",
                ));
            }
            files.push((pin.path.clone(), actual));
        }
        let configuration_sha256 = canonical_identity(&(
            config.configuration_identity().map_err(configuration)?,
            files,
        ))
        .map_err(configuration)?;
        let authentication_scope_sha256 = canonical_identity(&(
            &config.capability.provider.authentication_scope,
            self.environment(config)?,
        ))
        .map_err(configuration)?;
        Ok((configuration_sha256, authentication_scope_sha256))
    }
}
