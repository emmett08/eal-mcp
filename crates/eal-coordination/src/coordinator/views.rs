//! Detached status and export views of authoritative coordination state.
use super::*;
use std::{collections::BTreeSet, io::Write};

impl Coordinator {
    pub fn cancel(&self) -> Result<Value, String> {
        self.state.cancel()?;
        self.status()
    }
    pub fn status(&self) -> Result<Value, String> {
        let assessor = self.state.effect("worker:assessor")?;
        let reviewer = self.state.effect("worker:reviewer")?;
        let workers = [
            ("assessor", assessor.as_ref()),
            ("reviewer", reviewer.as_ref()),
        ];
        let mut answer = Value::Null;
        let mut status = if self.state.cancelled()? {
            "cancelled"
        } else if workers.iter().any(|(_, worker)| {
            worker.is_some_and(|worker| {
                worker["status"] == "committed" && worker["result"]["accepted"] != true
            })
        }) {
            "failed"
        } else if let Some(worker) = reviewer
            .as_ref()
            .filter(|worker| worker["status"] == "committed")
        {
            answer = worker["result"]["response"]["answer"].clone();
            if answer["conclusion"] == "assessed" {
                "completed"
            } else {
                "completed_unassessed"
            }
        } else if unix_seconds()? >= number(&self.config, "deadline")? {
            "expired"
        } else if workers
            .iter()
            .any(|(_, worker)| worker.is_some_and(|worker| worker["status"] == "unknown"))
        {
            "awaiting_worker_result"
        } else if assessor.is_some() {
            "ready_reviewer"
        } else {
            "ready_assessor"
        };
        let initial = self.state.effect("initial")?;
        if initial
            .as_ref()
            .is_some_and(|effect| effect["acknowledged"] != true)
            && !["cancelled", "expired"].contains(&status)
        {
            status = "delivery_pending";
        }
        if !["cancelled", "expired"].contains(&status) {
            let finished = self
                .state
                .events()?
                .iter()
                .filter(|event| event["kind"] == "transport_finished")
                .filter_map(|event| event["data"]["call_id"].as_str().map(str::to_owned))
                .collect::<BTreeSet<_>>();
            if workers.iter().any(|(_, worker)| {
                worker.is_some_and(|worker| {
                    worker["status"] == "committed"
                        && (worker["acknowledged"] != true
                            || !finished
                                .contains(worker["payload"]["call_id"].as_str().unwrap_or("")))
                })
            }) {
                status = "delivery_pending";
            }
        }
        let summaries=workers.into_iter().map(|(role,worker)|{
            (role.to_owned(),worker.map(|worker|json!({"state":worker["status"],"call_id":worker["payload"]["call_id"],"execution_mode":worker["payload"]["execution_mode"],"execution_id":worker["result"]["execution_id"],"outcome":worker["result"]["outcome"]})).unwrap_or(Value::Null))
        }).collect::<serde_json::Map<_,_>>();
        Ok(
            json!({"schema":"EAL/coordination-status/1","run_id":self.config["run_id"],"status":status,"task_digest":self.anchor["task_digest"],"deadline":self.config["deadline"],"workers":summaries,"answer":answer,"calls":self.state.calls()?}),
        )
    }
    pub fn export(&self, path: impl AsRef<Path>) -> Result<Value, String> {
        let value = json!({"schema":"EAL/coordination-export/1","config":self.config,"status":self.status()?,"effects":self.state.effects()?,"events":self.state.events()?,"transport":self.bus.status(string(&self.config,"run_id")?)?,
            "limits":["Trusted local host; shared filesystem permissions are not role isolation","Worker external execution receipts are host assertions, not authenticated provider receipts","Assessment times belong to individual MCP results; replay does not establish a current decision"]});
        let path = path.as_ref();
        if let Some(parent) = path.parent().filter(|path| !path.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let mut options = std::fs::OpenOptions::new();
        options.write(true).create(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600).custom_flags(
                (rustix::fs::OFlags::NOFOLLOW | rustix::fs::OFlags::NONBLOCK).bits() as i32,
            );
        }
        let mut file = options.open(path).map_err(|e| e.to_string())?;
        if !file.metadata().map_err(|e| e.to_string())?.is_file() {
            return Err("Export requires a regular file".into());
        }
        let mut bytes = serde_json::to_vec_pretty(&value).map_err(|e| e.to_string())?;
        bytes.push(b'\n');
        file.set_len(0).map_err(|e| e.to_string())?;
        file.write_all(&bytes).map_err(|e| e.to_string())?;
        Ok(value)
    }
}
