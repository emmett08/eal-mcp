//! Strict process protocol for the visualiser's explicitly selected native solver.
use eal_core::{ExecutionLimits, methods::MethodError};
use serde::Deserialize;
use serde_json::{Value, json};
use std::{
    ffi::OsString,
    fs::File,
    io::{self, Read, Write},
    path::{Path, PathBuf},
    process::ExitCode,
};

const MAX_REQUEST_BYTES: usize = 64 * 1024;
const MAX_RESPONSE_BYTES: usize = 4 * 1024 * 1024;
const MAX_LIMITS_BYTES: usize = 16 * 1024;
const MAX_ERROR_CHARACTERS: usize = 1800;

#[derive(Debug)]
enum Failure {
    Invalid(String),
    Incomplete(String),
    Unavailable(String),
}

impl Failure {
    fn code(&self) -> &'static str {
        match self {
            Self::Invalid(_) => "invalid_request",
            Self::Incomplete(_) => "incomplete",
            Self::Unavailable(_) => "unavailable",
        }
    }

    fn exit(&self) -> u8 {
        match self {
            Self::Invalid(_) => 2,
            Self::Incomplete(_) => 1,
            Self::Unavailable(_) => 3,
        }
    }

    fn message(&self) -> &str {
        match self {
            Self::Invalid(message) | Self::Incomplete(message) | Self::Unavailable(message) => {
                message
            }
        }
    }
}

impl From<MethodError> for Failure {
    fn from(error: MethodError) -> Self {
        match error {
            MethodError::Invalid(message) => Self::Invalid(message),
            MethodError::Incomplete(message) => Self::Incomplete(message),
        }
    }
}

fn arguments(mut arguments: impl Iterator<Item = OsString>) -> Result<Option<PathBuf>, Failure> {
    match arguments.next() {
        None => Ok(None),
        Some(flag) if flag == "--limits" => {
            let path = arguments
                .next()
                .filter(|path| !path.is_empty())
                .ok_or_else(|| Failure::Invalid("--limits requires a file path".into()))?;
            if arguments.next().is_some() {
                return Err(Failure::Invalid("Only --limits FILE is supported".into()));
            }
            Ok(Some(PathBuf::from(path)))
        }
        Some(_) => Err(Failure::Invalid("Only --limits FILE is supported".into())),
    }
}

fn io_failure(error: io::Error) -> Failure {
    Failure::Unavailable(error.to_string())
}

/// Canonicalise an operator-selected path, then reject non-regular descriptors.
/// NONBLOCK prevents a FIFO from delaying the request until a writer appears.
#[cfg(unix)]
fn open_limits(path: &Path) -> Result<File, Failure> {
    use rustix::fs::{Mode, OFlags, open};
    let descriptor = open(
        path,
        OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC,
        Mode::empty(),
    )
    .map_err(io::Error::from)
    .map_err(io_failure)?;
    Ok(File::from(descriptor))
}

#[cfg(not(unix))]
fn open_limits(path: &Path) -> Result<File, Failure> {
    if std::fs::symlink_metadata(path)
        .map_err(io_failure)?
        .file_type()
        .is_symlink()
    {
        return Err(Failure::Unavailable(
            "Limits file changed while opening".into(),
        ));
    }
    File::open(path).map_err(io_failure)
}

fn load_limits(path: Option<&Path>) -> Result<ExecutionLimits, Failure> {
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Document {
        limits: ExecutionLimits,
    }
    let Some(path) = path else {
        return Ok(ExecutionLimits::default());
    };
    let path = path.canonicalize().map_err(io_failure)?;
    let file = open_limits(&path)?;
    if !file.metadata().map_err(io_failure)?.is_file() {
        return Err(Failure::Invalid("Limits must be a regular file".into()));
    }
    let mut bytes = Vec::new();
    file.take(MAX_LIMITS_BYTES as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(io_failure)?;
    if bytes.len() > MAX_LIMITS_BYTES {
        return Err(Failure::Invalid(
            "Limits configuration exceeds 16 KiB".into(),
        ));
    }
    let text = std::str::from_utf8(&bytes).map_err(|error| Failure::Invalid(error.to_string()))?;
    let document: Document =
        toml::from_str(text).map_err(|error| Failure::Invalid(error.to_string()))?;
    document.limits.validate().map_err(Failure::Invalid)?;
    Ok(document.limits)
}

fn read_request(reader: impl Read) -> Result<Vec<u8>, Failure> {
    let mut bytes = Vec::new();
    reader
        .take(MAX_REQUEST_BYTES as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(io_failure)?;
    if bytes.len() > MAX_REQUEST_BYTES {
        return Err(Failure::Invalid(
            "Theory request exceeds the 64 KiB limit".into(),
        ));
    }
    Ok(bytes)
}

fn solve_request(bytes: &[u8], limits: &ExecutionLimits) -> Result<Value, Failure> {
    let text = std::str::from_utf8(bytes).map_err(|error| Failure::Invalid(error.to_string()))?;
    let wire_limits = ExecutionLimits {
        record_bytes: limits.record_bytes.min(MAX_REQUEST_BYTES),
        json_depth: limits.json_depth.min(eal_core::json::MAX_JSON_DEPTH),
        ..limits.clone()
    };
    let payload = eal_core::json::parse(text, &wire_limits).map_err(Failure::Invalid)?;
    if !payload.as_object().is_some_and(|object| {
        object.len() == 1 && object.get("theory").is_some_and(Value::is_object)
    }) {
        return Err(Failure::Invalid(
            "Request must contain exactly one theory object".into(),
        ));
    }
    let contract = eal_aspic::aspic_contract();
    let input =
        eal_core::canonical::canonical_json_bytes(&payload, limits).map_err(Failure::Invalid)?;
    if input.len() > contract.max_input_bytes {
        return Err(Failure::Invalid("Method input exceeds byte limit".into()));
    }
    // The registered native callback preserves typed exhaustion and the complete
    // formal response cap. Its timeout metadata does not supervise this process.
    let implementation = contract
        .implementation
        .as_ref()
        .ok_or_else(|| Failure::Unavailable("Native ASPIC implementation is unavailable".into()))?;
    let formal = implementation.call(&payload, limits)?;
    eal_aspic::export_aspic_view(
        &json!({"theory":payload["theory"],"formal":formal,"profile":eal_aspic::METHOD}),
        limits,
    )
    .map_err(Failure::Invalid)
}

struct BoundedOutput {
    bytes: Vec<u8>,
    maximum: usize,
    exceeded: bool,
}

impl Write for BoundedOutput {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if bytes.len() > self.maximum.saturating_sub(self.bytes.len()) {
            self.exceeded = true;
            return Err(io::Error::other(
                "ASPIC view exceeds the 4 MiB output limit",
            ));
        }
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn encode_view(view: &Value) -> Result<Vec<u8>, Failure> {
    let mut output = BoundedOutput {
        bytes: Vec::new(),
        maximum: MAX_RESPONSE_BYTES - 1, // The consumer cap includes the newline.
        exceeded: false,
    };
    if let Err(error) = serde_json::to_writer(&mut output, view) {
        return Err(if output.exceeded {
            Failure::Incomplete("ASPIC view exceeds the 4 MiB output limit".into())
        } else {
            Failure::Unavailable(error.to_string())
        });
    }
    output.bytes.push(b'\n');
    Ok(output.bytes)
}

fn execute() -> Result<(), Failure> {
    let path = arguments(std::env::args_os().skip(1))?;
    // No ambient configuration is read: only the explicit operator flag applies.
    let limits = load_limits(path.as_deref())?;
    let request = read_request(io::stdin().lock())?;
    let view = solve_request(&request, &limits)?;
    let output = encode_view(&view)?;
    let mut stdout = io::stdout().lock();
    stdout.write_all(&output).map_err(io_failure)?;
    stdout.flush().map_err(io_failure)
}

pub fn main() -> ExitCode {
    match execute() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            let message: String = error.message().chars().take(MAX_ERROR_CHARACTERS).collect();
            let envelope = json!({"error":{"code":error.code(),"message":message}});
            // Serialization is infallible for this bounded string-only envelope.
            let mut bytes = serde_json::to_vec(&envelope).expect("bounded error envelope");
            bytes.push(b'\n');
            let _ = io::stderr().lock().write_all(&bytes);
            ExitCode::from(error.exit())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn response_cap_includes_the_newline_and_never_returns_partial_bytes() {
        let exact = Value::String("x".repeat(MAX_RESPONSE_BYTES - 3));
        assert_eq!(encode_view(&exact).unwrap().len(), MAX_RESPONSE_BYTES);
        let oversized = Value::String("x".repeat(MAX_RESPONSE_BYTES - 2));
        assert!(matches!(
            encode_view(&oversized),
            Err(Failure::Incomplete(_))
        ));
    }
}
