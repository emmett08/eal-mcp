//! Retain synthetic workspaces and run installed commands through the native supervisor.
use super::{Options, Result, ensure, parse_json, write_json};
use eal_runtime::process::{CommandRequest, ProcessOutcome, Supervisor};
use serde_json::{Value, json};
use std::{
    cell::RefCell,
    collections::BTreeMap,
    path::{Path, PathBuf},
    time::Duration,
};

pub const SOURCE: &str = "language \"EAL/4\"\nenvironment lab {\n require site == \"bench\"\n}\ntool reader {\n version \"1\"\n}\nevidence reading {\n tool reader\n kind test\n environment lab\n max_age 3600\n input {\"sample\":1}\n require ok == true\n}\nreasoning measured {\n method \"structured/1\"\n rationale \"The supplied report supports the selected test result.\"\n}\nclaim works {\n statement \"The supplied test passed at this bench.\"\n environment lab\n}\nargument result = [evidence reading] via measured => works\n";
pub const TOKEN: &str = "synthetic-installed-operator-token.ABC_123~";
pub const TOKEN_ENV: &str = "EAL_SMOKE_OPERATOR_TOKEN";
pub const MAX_OUTPUT: usize = 16 * 1024 * 1024;
pub struct Harness {
    pub prefix: PathBuf,
    pub root: PathBuf,
    pub now: String,
    pub profile: &'static str,
    pub environment: BTreeMap<String, String>,
    pub launches: RefCell<Vec<Value>>,
    supervisor: Supervisor,
}
impl Harness {
    pub fn new(options: &Options, profile: &'static str, required: &[&str]) -> Result<Self> {
        let prefix = options
            .prefix
            .canonicalize()
            .map_err(|e| format!("--prefix requires an installed binary directory: {e}"))?;
        for name in required {
            let path = prefix.join(name);
            ensure(
                path.is_file(),
                format!("Missing installed companion {}", path.display()),
            )?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                ensure(
                    path.metadata()
                        .map_err(|e| e.to_string())?
                        .permissions()
                        .mode()
                        & 0o111
                        != 0,
                    format!("Installed companion is not executable {}", path.display()),
                )?;
            }
        }
        let root = match &options.workspace {
            Some(path) => {
                if path.is_absolute() {
                    path.clone()
                } else {
                    std::env::current_dir()
                        .map_err(|e| e.to_string())?
                        .join(path)
                }
            }
            None => {
                std::env::temp_dir().join(format!("eal-installed-smoke-{}", uuid::Uuid::new_v4()))
            }
        };
        if root.exists() {
            ensure(
                root.is_dir()
                    && std::fs::read_dir(&root)
                        .map_err(|e| e.to_string())?
                        .next()
                        .is_none(),
                "--workspace must be empty or new",
            )?;
        }
        std::fs::create_dir_all(&root).map_err(|e| e.to_string())?;
        let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
        std::fs::write(root.join("case.eal"), SOURCE).map_err(|e| e.to_string())?;
        write_json(&root.join("context.json"), &json!({"site":"bench"}))?;
        std::fs::write(
            root.join("tools.toml"),
            "[tools.reader]\nkind='json_file'\nversion='1'\npath='observation.json'\n",
        )
        .map_err(|e| e.to_string())?;
        write_json(
            &root.join("observation.json"),
            &json!({"observed_at":now,"context":{"site":"bench"},"request":{"tool":"reader","tool_version":"1","input":{"sample":1},"context":{"site":"bench"}},"value":{"ok":true,"secret":"collector-secret"}}),
        )?;
        let environment = BTreeMap::from([
            ("PATH".into(), "".into()),
            ("LANG".into(), "C.UTF-8".into()),
            ("NO_PROXY".into(), "*".into()),
            (TOKEN_ENV.into(), TOKEN.into()),
        ]);
        let supervisor = Supervisor::new(prefix.join("eal-process-supervisor"));
        Ok(Self {
            prefix,
            root,
            now,
            profile,
            environment,
            launches: RefCell::default(),
            supervisor,
        })
    }
    pub fn common(&self) -> Vec<String> {
        vec![
            "--workspace".into(),
            self.root.display().to_string(),
            "--registry".into(),
            self.root.join("tools.toml").display().to_string(),
            "--methods".into(),
            self.profile.into(),
        ]
    }
    pub fn process(
        &self,
        executable: &Path,
        arguments: &[String],
        request: Option<&Value>,
        code: i32,
    ) -> Result<ProcessOutcome> {
        let mut argv = vec![executable.display().to_string()];
        argv.extend_from_slice(arguments);
        let input = request
            .map(serde_json::to_vec)
            .transpose()
            .map_err(|e| e.to_string())?
            .unwrap_or_default();
        let outcome = self
            .supervisor
            .run(
                CommandRequest {
                    argv,
                    environment: self.environment.clone(),
                    cwd: self.root.clone(),
                    input,
                    timeout: Duration::from_secs(60),
                    max_output_bytes: MAX_OUTPUT,
                },
                None,
            )
            .map_err(|e| e.to_string())?;
        self.launches.borrow_mut().push(json!({"executable":executable,"operation":arguments.first(),"returncode":outcome.returncode}));
        ensure(
            outcome.execution_error.is_none() && outcome.returncode == code,
            format!(
                "{} {:?} failed with {} {:?}: {} {}",
                executable.display(),
                arguments,
                outcome.returncode,
                outcome.execution_error,
                String::from_utf8_lossy(&outcome.stdout),
                String::from_utf8_lossy(&outcome.stderr)
            ),
        )?;
        Ok(outcome)
    }
    pub fn call(
        &self,
        executable: &str,
        arguments: &[String],
        request: Option<&Value>,
        code: i32,
        error: bool,
    ) -> Result<Value> {
        let outcome = self.process(&self.prefix.join(executable), arguments, request, code)?;
        let bytes = if error {
            ensure(outcome.stdout.is_empty(), "Error command wrote stdout")?;
            &outcome.stderr
        } else {
            &outcome.stdout
        };
        let value = parse_json(bytes, MAX_OUTPUT)
            .map_err(|e| format!("{executable} did not return exactly one JSON object: {e}"))?;
        ensure(
            value.is_object(),
            format!("{executable} response must be an object"),
        )?;
        if error {
            ensure(
                value["error"].is_object(),
                "Expected structured coordination error",
            )?;
        }
        Ok(value)
    }
    pub fn cli(&self, args: &[&str]) -> Result<Value> {
        let mut arguments = self.common();
        arguments.extend(args.iter().map(|s| s.to_string()));
        self.call("eal", &arguments, None, 0, false)
    }
    pub fn host(&self, request: &Value, args: &[&str], code: i32) -> Result<Value> {
        let mut arguments = self.common();
        arguments.extend(["--timeout".into(), "15".into()]);
        arguments.extend(args.iter().map(|s| s.to_string()));
        self.call("eal-host", &arguments, Some(request), code, false)
    }
    pub fn coordinate(
        &self,
        operation: &str,
        directory: &Path,
        args: &[String],
        code: i32,
        error: bool,
    ) -> Result<Value> {
        let mut arguments = vec![
            operation.into(),
            "--run".into(),
            directory.display().to_string(),
        ];
        arguments.extend_from_slice(args);
        self.call("eal-coordinate", &arguments, None, code, error)
    }
    pub fn finish(&self, mut result: Value) -> Result<Value> {
        result["result"] = json!("passed");
        result["prefix"] = json!(self.prefix);
        result["workspace"] = json!(self.root);
        result["observed_at"] = json!(self.now);
        result["profile"] = json!(self.profile);
        result["child_path"] = json!("");
        write_json(&self.root.join("smoke-result.json"), &result)?;
        Ok(result)
    }
}
