//! Immutable wire frames and durable reservations for one external-host session.
use crate::{
    ledger::{EffectStatus, canonical_json},
    state::State,
};
use eal_coordination_contracts::{Dispatch, MAX_FRAME_BYTES, PeerValue, Session, SessionSpec};
use eal_runtime::private_sqlite::ensure_immutable_file;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    fs::OpenOptions,
    io::{Read, Write},
    path::{Path, PathBuf},
};

const RUN_SCHEMA: &str = "EAL/debate-session-run/1";
const MAX_FRAMES: usize = 64;
// Opaque frame contents live in separate bounded files, keeping this index small.
const MAX_INDEX_BYTES: u64 = 16 * 1024 * 1024;

pub(super) struct Journal {
    directory: PathBuf,
    pub state: State,
    frames: usize,
}
impl Journal {
    pub fn new(directory: &Path) -> Result<Self, String> {
        if std::fs::symlink_metadata(directory)
            .is_ok_and(|metadata| metadata.file_type().is_symlink())
        {
            return Err("debate session directory must not be a symlink".into());
        }
        std::fs::create_dir_all(directory).map_err(|error| error.to_string())?;
        let directory = directory
            .canonicalize()
            .map_err(|error| error.to_string())?;
        if std::fs::read_dir(&directory)
            .map_err(|error| error.to_string())?
            .next()
            .is_some()
        {
            return Err("debate sessions require an empty directory; uncertain calls are never automatically resumed".into());
        }
        // Persist ownership before opening SQLite so concurrent starts have one winner.
        let mut ownership_options = OpenOptions::new();
        ownership_options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            ownership_options.mode(0o600).custom_flags(
                (rustix::fs::OFlags::NOFOLLOW | rustix::fs::OFlags::NONBLOCK).bits() as i32,
            );
        }
        let mut ownership = ownership_options.open(directory.join("session.owner"))
            .map_err(|_| "debate session directory already has an owner; automatic reexecution is disabled")?;
        ownership
            .write_all(uuid::Uuid::new_v4().to_string().as_bytes())
            .map_err(|error| error.to_string())?;
        ownership.sync_all().map_err(|error| error.to_string())?;
        let state = State::new(directory.join("debate.sqlite"))?;
        Ok(Self {
            directory,
            state,
            frames: 0,
        })
    }
    pub fn freeze(&self, spec: &SessionSpec, calls: usize) -> Result<(), String> {
        let value = serde_json::to_value(spec).map_err(|error| error.to_string())?;
        let spec_sha256 = digest(canonical_json(&value)?.as_bytes());
        self.state.initialize(&json!({"schema":RUN_SCHEMA,"run_id":format!("debate-{}",uuid::Uuid::new_v4().simple()),"spec":value,"spec_sha256":spec_sha256,"expected_calls":calls,"created_at":eal_runtime::store::utc_now()}))?;
        self.state.event(
            "session_admitted",
            &json!({"spec_sha256":spec_sha256,"expected_calls":calls}),
        )
    }
    /// Capture exact input before admission and exact output before publication.
    pub fn frame(
        &mut self,
        direction: &str,
        bytes: &[u8],
        complete: bool,
    ) -> Result<Value, String> {
        if self.frames >= MAX_FRAMES || bytes.len() > MAX_FRAME_BYTES + 1 {
            return Err("session journal frame bound exhausted".into());
        }
        let name = format!("frame-{:04}-{direction}.bin", self.frames);
        ensure_immutable_file(self.directory.join(&name), bytes)?;
        let receipt = json!({"index":self.frames,"direction":direction,"path":name,"bytes":bytes.len(),"sha256":digest(bytes),"complete":complete});
        self.state.event("session_frame", &receipt)?;
        self.frames += 1;
        Ok(receipt)
    }
    pub fn reserve(&self, dispatch: &Dispatch, frame: &Value) -> Result<(), String> {
        for call in &dispatch.calls {
            let payload =
                json!({"participant":call.participant,"round":call.round,"dispatch_frame":frame});
            if self
                .state
                .reserve_effect(&key(call.round, call.participant), &payload)?
                .status
                != EffectStatus::New
            {
                return Err(
                    "session dispatch already reserved; automatic reexecution is disabled".into(),
                );
            }
        }
        Ok(())
    }
    pub fn accept(&self, responses: &[PeerValue], frame: &Value) -> Result<(), String> {
        for response in responses {
            self.state.complete_effect(&key(response.round,response.participant), &json!({"participant":response.participant,"round":response.round,"payload_sha256":digest(&serde_json::to_vec(&response.payload).map_err(|error|error.to_string())?),"response_frame":frame}), &[])?;
        }
        self.state.event(
            "round_completed",
            &json!({"round":responses.first().map(|value|value.round),"calls":responses.len()}),
        )
    }
    pub fn completed(&self, rounds: usize, calls: usize) -> Result<(), String> {
        self.state
            .event("session_completed", &json!({"rounds":rounds,"calls":calls}))
    }
    pub fn failed(&self, message: &str) -> Result<(), String> {
        self.state
            .event("session_failed", &json!({"message":message}))
    }
}
fn key(round: usize, participant: usize) -> String {
    format!("debate:{round}:{participant}")
}
fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn open(directory: &Path) -> Result<(PathBuf, State), String> {
    if std::fs::symlink_metadata(directory).is_ok_and(|metadata| metadata.file_type().is_symlink())
    {
        return Err("debate session directory must not be a symlink".into());
    }
    let directory = directory
        .canonicalize()
        .map_err(|error| error.to_string())?;
    let database = directory.join("debate.sqlite");
    let metadata =
        std::fs::symlink_metadata(&database).map_err(|_| "no debate session journal exists")?;
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err("debate session journal must be a regular file".into());
    }
    if metadata.len() > MAX_INDEX_BYTES {
        return Err("debate session journal index exceeds its byte bound".into());
    }
    let state = State::new(database)?;
    Ok((directory, state))
}
fn read_frame(directory: &Path, frame: &Value) -> Result<(), String> {
    let index = frame["index"]
        .as_u64()
        .filter(|index| *index < MAX_FRAMES as u64)
        .ok_or("invalid journal frame index")?;
    let direction = frame["direction"]
        .as_str()
        .filter(|value| ["in", "out"].contains(value))
        .ok_or("invalid journal direction")?;
    let expected = format!("frame-{index:04}-{direction}.bin");
    if frame["path"].as_str() != Some(&expected) {
        return Err("invalid journal frame path".into());
    }
    let size = frame["bytes"]
        .as_u64()
        .filter(|value| *value <= MAX_FRAME_BYTES as u64 + 1)
        .ok_or("invalid journal frame byte count")?;
    let path = directory.join(expected);
    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(
            (rustix::fs::OFlags::NOFOLLOW | rustix::fs::OFlags::NONBLOCK).bits() as i32,
        );
    }
    let file = options.open(path).map_err(|error| error.to_string())?;
    if !file
        .metadata()
        .map_err(|error| error.to_string())?
        .is_file()
    {
        return Err("journal frame must be a regular file".into());
    }
    let mut bytes = Vec::new();
    file.take(MAX_FRAME_BYTES as u64 + 2)
        .read_to_end(&mut bytes)
        .map_err(|error| error.to_string())?;
    if bytes.len() as u64 != size || frame["sha256"].as_str() != Some(digest(&bytes).as_str()) {
        return Err("immutable journal frame content differs".into());
    }
    Ok(())
}
fn snapshot(directory: &Path) -> Result<Value, String> {
    let (directory, state) = open(directory)?;
    let events = state.events()?;
    let frames = events
        .iter()
        .filter(|event| event["kind"] == "session_frame")
        .collect::<Vec<_>>();
    if frames.len() > MAX_FRAMES {
        return Err("journal contains too many frames".into());
    }
    for frame in &frames {
        read_frame(&directory, &frame["data"])?;
    }
    let config = match state.config() {
        Ok(config) => Some(config),
        Err(error)
            if error == "No coordination run exists in this directory"
                && !events
                    .iter()
                    .any(|event| event["kind"] == "session_admitted") =>
        {
            None
        }
        Err(error) => return Err(error),
    };
    let expected = if let Some(config) = &config {
        if config["schema"] != RUN_SCHEMA {
            return Err("unsupported debate journal schema".into());
        }
        let spec: SessionSpec =
            serde_json::from_value(config["spec"].clone()).map_err(|error| error.to_string())?;
        let session = Session::new(spec).map_err(|error| error.to_string())?;
        if config["expected_calls"].as_u64() != Some(session.total_calls() as u64)
            || config["spec_sha256"].as_str()
                != Some(digest(canonical_json(&config["spec"])?.as_bytes()).as_str())
        {
            return Err("frozen session configuration identity differs".into());
        }
        session.total_calls()
    } else {
        0
    };
    let effects = state.effects()?;
    let committed = effects
        .iter()
        .filter(|effect| effect["status"] == "committed")
        .count();
    let reserved = effects.len().saturating_sub(committed);
    let failed = events.iter().any(|event| event["kind"] == "session_failed");
    let completed = events
        .iter()
        .any(|event| event["kind"] == "session_completed");
    let status = if failed {
        "failed"
    } else if completed && committed == expected && reserved == 0 {
        "completed"
    } else if reserved > 0 {
        "awaiting_results"
    } else {
        "incomplete"
    };
    Ok(
        json!({"schema":"EAL/debate-session-export/1","config":config,"status":{"schema":"EAL/debate-session-status/1","status":status,"expected_calls":expected,"committed_calls":committed,"uncertain_calls":reserved,"frames":frames.len(),"automatic_reexecution":false},"effects":effects,"events":events,"frames_directory":directory}),
    )
}
pub(super) fn status(directory: &Path) -> Result<Value, String> {
    Ok(snapshot(directory)?["status"].clone())
}
pub(super) fn export(directory: &Path, output: Option<&Path>) -> Result<Value, String> {
    let value = snapshot(directory)?;
    if let Some(output) = output {
        let bytes = serde_json::to_vec_pretty(&value).map_err(|error| error.to_string())?;
        ensure_immutable_file(output, &bytes)?;
    }
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Barrier};

    #[test]
    fn concurrent_session_starts_have_one_persistent_owner() {
        let directory =
            std::env::temp_dir().join(format!("eal-session-owner-{}", uuid::Uuid::new_v4()));
        let barrier = Arc::new(Barrier::new(8));
        let handles = (0..8)
            .map(|_| {
                let directory = directory.clone();
                let barrier = Arc::clone(&barrier);
                std::thread::spawn(move || {
                    barrier.wait();
                    Journal::new(&directory).is_ok()
                })
            })
            .collect::<Vec<_>>();
        let successful = handles
            .into_iter()
            .map(|handle| handle.join().unwrap())
            .filter(|successful| *successful)
            .count();
        assert_eq!(successful, 1);
        assert!(Journal::new(&directory).is_err());
        let _ = std::fs::remove_dir_all(directory);
    }
}
