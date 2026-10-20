//! Immutable host-configured method commands and supervised JSON callbacks.
pub(crate) mod protocol;
use crate::process::{CommandRequest, ExecutionError, StartupHandshake, Supervisor};
use eal_core::{
    ExecutionLimits, canonical,
    methods::{MethodContract, MethodError, MethodRegistry, NativeMethodIdentity},
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, io::Read, path::PathBuf, time::Duration};

#[derive(Clone, Debug)]
pub struct WorkerCommand {
    argv: Vec<String>,
    environment: BTreeMap<String, String>,
    cwd: PathBuf,
    executable_sha256: String,
}
impl WorkerCommand {
    pub fn new(
        mut argv: Vec<String>,
        environment: BTreeMap<String, String>,
        cwd: PathBuf,
    ) -> Result<Self, MethodError> {
        if argv.is_empty()
            || argv.len() > 256
            || argv.iter().any(|s| s.contains('\0') || s.len() > 8192)
            || !PathBuf::from(&argv[0]).is_absolute()
            || !cwd.is_absolute()
            || !cwd.is_dir()
            || cwd.to_str().is_none()
        {
            return Err(
                "Method command requires absolute executable/cwd and bounded explicit argv".into(),
            );
        }
        if environment.len() > 256
            || environment.iter().any(|(k, v)| {
                k.is_empty()
                    || k.contains(['=', '\0'])
                    || v.contains('\0')
                    || k.len() + v.len() > 16384
            })
        {
            return Err("Invalid explicit method environment".into());
        }
        argv[0] = std::fs::canonicalize(&argv[0])
            .map_err(|e| e.to_string())?
            .to_str()
            .ok_or("Method executable path must be UTF-8")?
            .to_owned();
        let executable_sha256 = artifact(&argv[0])?;
        Ok(Self {
            argv,
            environment,
            cwd,
            executable_sha256,
        })
    }
    fn check(&self) -> Result<(), MethodError> {
        if artifact(&self.argv[0])? != self.executable_sha256 {
            return Err("Method executable differs from its registered artifact identity".into());
        }
        Ok(())
    }
    fn describe(&self) -> Value {
        json!({"argv":self.argv,"environment":self.environment,"cwd":self.cwd,"executable_sha256":self.executable_sha256})
    }
}
fn artifact(path: &str) -> Result<String, MethodError> {
    let resolved = std::fs::canonicalize(path).map_err(|e| e.to_string())?;
    if !resolved.metadata().map_err(|e| e.to_string())?.is_file() {
        return Err("Method executable must be a regular file".into());
    }
    let mut options = std::fs::OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(
            (rustix::fs::OFlags::NONBLOCK | rustix::fs::OFlags::NOFOLLOW).bits() as i32,
        );
    }
    let mut file = options
        .open(&resolved)
        .map_err(|e| MethodError::Invalid(format!("Cannot open method executable: {e}")))?;
    let before = file.metadata().map_err(|e| e.to_string())?;
    if !before.is_file() {
        return Err("Method executable must be a regular file".into());
    }
    let mut hash = Sha256::new();
    let mut chunk = [0u8; 32768];
    let mut remaining = before.len();
    while remaining > 0 {
        let count = remaining.min(chunk.len() as u64) as usize;
        let n = file.read(&mut chunk[..count]).map_err(|e| e.to_string())?;
        if n == 0 {
            return Err("Method executable changed while its identity was read".into());
        }
        hash.update(&chunk[..n]);
        remaining -= n as u64;
    }
    let after = file.metadata().map_err(|e| e.to_string())?;
    if before.len() != after.len() || before.modified().ok() != after.modified().ok() {
        return Err("Method executable changed while its identity was read".into());
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if (
            before.dev(),
            before.ino(),
            before.ctime(),
            before.ctime_nsec(),
        ) != (after.dev(), after.ino(), after.ctime(), after.ctime_nsec())
        {
            return Err("Method executable changed while its identity was read".into());
        }
    }
    Ok(format!("{:x}", hash.finalize()))
}

#[derive(Clone, Debug)]
pub struct MethodExecutor {
    supervisor: Supervisor,
    worker: WorkerCommand,
    supervisor_sha256: String,
}
impl MethodExecutor {
    pub fn new(supervisor: Supervisor, worker: WorkerCommand) -> Result<Self, MethodError> {
        if !supervisor.executable().is_absolute() {
            return Err("Method supervisor path must be absolute".into());
        }
        worker.check()?;
        let path = supervisor
            .executable()
            .to_str()
            .ok_or("Method supervisor path must be UTF-8")?;
        let supervisor_sha256 = artifact(path)?;
        Ok(Self {
            supervisor,
            worker,
            supervisor_sha256,
        })
    }
    pub fn supervise_registry(&self, registry: &MethodRegistry) -> Result<MethodRegistry, String> {
        let mut result = registry.clone();
        for (id, _) in registry.describe().as_object().unwrap() {
            if registry.get(id).unwrap().describe()["execution"] == "host_registered_executor" {
                continue;
            }
            if [
                "structured/1",
                "deductive/1",
                "inductive/1",
                "abductive/1",
                "causal/1",
                "counterfactual/1",
                "analogical/1",
                "temporal/1",
            ]
            .contains(&id.as_str())
            {
                continue;
            }
            if !eal_core::methods::optional::IDENTIFIERS.contains(&id.as_str())
                && id != "argumentation/aspic/2"
            {
                return Err(format!(
                    "Method {id:?} requires an operator-registered worker command"
                ));
            }
            let contract = registry.get(id).unwrap().clone();
            let executor = self.clone();
            let identity = self.identity(&contract, None).map_err(|e| e.to_string())?;
            result = result.with_executor(id, identity, move |payload, limits| {
                executor.execute(&contract, payload, limits)
            })?;
        }
        Ok(result)
    }
    pub fn register_command(
        &self,
        registry: &MethodRegistry,
        contract: MethodContract,
        command: WorkerCommand,
    ) -> Result<MethodRegistry, String> {
        let identity = self
            .identity(&contract, Some(&command))
            .map_err(|e| e.to_string())?;
        let executor = self.clone();
        let id = contract.identifier.clone();
        let original = contract.clone();
        registry
            .with_method(contract)?
            .with_executor(&id, identity, move |payload, limits| {
                executor.execute_command(&original, payload, limits, Some(&command))
            })
    }
    fn identity(
        &self,
        contract: &MethodContract,
        command: Option<&WorkerCommand>,
    ) -> Result<NativeMethodIdentity, MethodError> {
        let description = json!({"protocol":protocol::VERSION,"worker":self.worker.describe(),"supervisor":{"path":self.supervisor.executable(),"sha256":self.supervisor_sha256},"command":command.map(WorkerCommand::describe),"contract":contract.describe(),"runtime_source":include_str!("methods.rs"),"protocol_source":include_str!("methods/protocol.rs"),"worker_source":include_str!("method_worker.rs")});
        Ok(NativeMethodIdentity {
            version: format!("eal-rust-supervised-method-{}", env!("CARGO_PKG_VERSION")),
            artifact_sha256: canonical::canonical_digest(
                &description,
                &protocol::wire_limits(protocol::MAX_REQUEST_BYTES),
            )?,
        })
    }
    pub fn execute(
        &self,
        contract: &MethodContract,
        payload: &Value,
        limits: &ExecutionLimits,
    ) -> Result<Value, MethodError> {
        self.execute_command(contract, payload, limits, None)
    }
    fn execute_command(
        &self,
        contract: &MethodContract,
        payload: &Value,
        limits: &ExecutionLimits,
        command: Option<&WorkerCommand>,
    ) -> Result<Value, MethodError> {
        limits.validate()?;
        MethodRegistry::new([contract.clone()])?;
        self.worker.check()?;
        if artifact(
            self.supervisor
                .executable()
                .to_str()
                .ok_or("Method supervisor path must be UTF-8")?,
        )? != self.supervisor_sha256
        {
            return Err("Method supervisor differs from its registered artifact identity".into());
        }
        if let Some(command) = command {
            command.check()?;
        }
        if canonical::canonical_json_bytes(payload, limits)?.len() > contract.max_input_bytes {
            return Err("Method input exceeds byte limit".into());
        }
        let errors = eal_core::methods::schema_errors_with_limits(
            payload,
            &contract.input_schema,
            "$",
            limits,
        );
        if !errors.is_empty() {
            return Err(errors.join("; ").into());
        }
        let request = protocol::Request {
            protocol: protocol::VERSION.into(),
            method: contract.identifier.clone(),
            contract: contract.describe(),
            payload: payload.clone(),
            limits: limits.clone(),
        };
        let input = protocol::encode(
            &serde_json::to_value(request).map_err(|e| e.to_string())?,
            protocol::MAX_REQUEST_BYTES,
        )?;
        let mut argv = self.worker.argv.clone();
        argv.extend([
            "--timeout-seconds".into(),
            contract.timeout_seconds.to_string(),
            "--max-memory-bytes".into(),
            contract.max_memory_bytes.to_string(),
        ]);
        if let Some(command) = command {
            argv.push("--exec".into());
            argv.extend(command.argv.clone());
        }
        let selected = command.unwrap_or(&self.worker);
        let outcome = self
            .supervisor
            .run_framed(
                CommandRequest {
                    argv,
                    environment: selected.environment.clone(),
                    cwd: selected.cwd.clone(),
                    input,
                    timeout: Duration::from_secs_f64(contract.timeout_seconds),
                    max_output_bytes: contract.max_output_bytes + protocol::READY.len() + 1,
                },
                None,
                StartupHandshake {
                    expected_first_stdout_line: protocol::READY.to_vec(),
                    startup_timeout: Duration::from_secs(5),
                },
            )
            .map_err(|e| MethodError::Invalid(e.to_string()))?;
        if let Some(error) = outcome.execution_error {
            return Err(match error {
                ExecutionError::Timeout => MethodError::Incomplete(
                    if outcome.startup_completed {
                        "Method execution exceeded its timeout"
                    } else {
                        "Method worker startup exceeded its timeout"
                    }
                    .into(),
                ),
                ExecutionError::OutputLimit => {
                    MethodError::Invalid("Method output exceeds byte limit".into())
                }
                ExecutionError::Cancelled => {
                    MethodError::Incomplete("Method execution was cancelled".into())
                }
                other => MethodError::Invalid(format!("Method worker failed: {other:?}")),
            });
        }
        if outcome.returncode != 0 {
            let message = format!(
                "Method worker exited without a successful result (status {})",
                outcome.returncode
            );
            return Err(if [-9, -24, -6].contains(&outcome.returncode) {
                MethodError::Incomplete(message)
            } else {
                MethodError::Invalid(message)
            });
        }
        let output = protocol::decode_response(&outcome.stdout, contract.max_output_bytes)?;
        let errors = eal_core::methods::schema_errors_with_limits(
            &output,
            &contract.output_schema,
            "$",
            limits,
        );
        if !errors.is_empty() {
            return Err(format!("Method output contract violation: {}", errors.join("; ")).into());
        }
        Ok(output)
    }
}
