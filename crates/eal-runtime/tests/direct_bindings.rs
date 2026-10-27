use eal_runtime::{
    acquisition::{self, ToolBinding, ToolRegistry},
    process::Supervisor,
};
use serde_json::{Value, json};
use std::{
    fs,
    io::{Read, Write},
    net::TcpListener,
    path::{Path, PathBuf},
};

struct Directory(PathBuf);
impl Directory {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!("eal-direct-{}", uuid::Uuid::new_v4()));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn manifest(&self, text: &str) -> ToolRegistry {
        fs::write(self.0.join("tools.toml"), text).unwrap();
        ToolRegistry::load(self.0.join("tools.toml")).unwrap()
    }
}
impl Drop for Directory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn acquire(
    binding: &ToolBinding,
    input: Value,
    workspace: &Path,
) -> Result<acquisition::AcquisitionResult, acquisition::AcquisitionError> {
    acquisition::acquire(
        binding,
        &json!({"input":input}),
        workspace,
        &[17; 32],
        &Supervisor::new(companion::path()),
        None,
    )
}

#[test]
fn direct_bindings_reject_mixed_transport_authority_and_invalid_mappings() {
    let valid = "[tools.read]\nkind='mcp_stdio'\nversion='1'\nargv=['python3','server.py']\n[tools.read.direct]\ntool='snapshot'\nvalue_pointer='/value'\n";
    assert!(ToolRegistry::from_toml(valid).is_ok());
    for text in [
        valid.replace("tool='snapshot'", "tool='snapshot'\nurl='https://example.com/mcp'"),
        valid.replace("value_pointer='/value'", "value_pointer='/bad~escape'"),
        valid.replace("kind='mcp_stdio'", "kind='command'"),
        valid.replace("tool='snapshot'", "tool='snapshot'\noutput_schema_sha256='0000000000000000000000000000000000000000000000000000000000000000'"),
        "[tools.read]\nkind='mcp_http'\nversion='1'\n[tools.read.direct]\ntool='read'\nurl='http://example.com/mcp'\n".into(),
        "[tools.read]\nkind='http_get'\nversion='1'\n[tools.read.direct]\nurl='https://example.com/data?credential=x'\n".into(),
    ] { assert!(ToolRegistry::from_toml(&text).is_err(), "{text}"); }
}

#[test]
fn manifest_anchors_retained_files_and_context_independently_of_run_directory() {
    let package = Directory::new();
    let run = Directory::new();
    let registry = package.manifest("[context]\nsite='lab'\n[tools.saved]\nkind='json_file'\nversion='1'\npath='reading.json'\n");
    assert_eq!(registry.context, json!({"site":"lab"}));
    fs::write(
        package.0.join("reading.json"),
        br#"{"value":false,"observed_at":"2026-01-02T03:04:05Z","request":{},"context":{}}"#,
    )
    .unwrap();
    let result = acquire(&registry.bindings["saved"], json!({}), &run.0).unwrap();
    let envelope: Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(envelope["value"], false);
    assert_eq!(envelope["observed_at"], "2026-01-02T03:04:05Z");
}

#[test]
fn commands_inherit_only_named_environment_and_run_at_manifest_root() {
    let package = Directory::new();
    let run = Directory::new();
    let registry = package.manifest(
        "[tools.read]\nkind='command'\nversion='1'\nargv=['/bin/sh','-c','printf %s \"$PWD\"']\n",
    );
    let binding = &registry.bindings["read"];
    assert!(binding.effective_environment().unwrap().is_empty());
    assert!(
        ToolBinding::command("read", "1", vec!["/bin/true".into()])
            .effective_environment()
            .unwrap()
            .is_empty()
    );
    let result = acquire(binding, json!({}), &run.0).unwrap();
    assert_eq!(
        String::from_utf8(result.stdout).unwrap(),
        package.0.canonicalize().unwrap().to_str().unwrap()
    );
}

#[test]
fn http_get_keeps_negative_values_original_clock_and_exact_integer() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let server = std::thread::spawn(move || {
        let (mut socket, _) = listener.accept().unwrap();
        let mut request = [0; 4096];
        let n = socket.read(&mut request).unwrap();
        let request = std::str::from_utf8(&request[..n]).unwrap();
        assert!(
            request.starts_with("GET /reading?name=a%26b HTTP/1.1"),
            "{request}"
        );
        let body = r#"{"payload":{"passed":false,"count":9007199254740993},"time":"2026-01-02T03:04:05Z"}"#;
        write!(socket,"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",body.len()).unwrap();
    });
    let package = Directory::new();
    let registry = package.manifest(&format!("[tools.read]\nkind='http_get'\nversion='1'\n[tools.read.direct]\nurl='http://{address}/reading'\nvalue_pointer='/payload'\nobserved_at_pointer='/time'\n"));
    let result = acquire(
        &registry.bindings["read"],
        json!({"name":"a&b"}),
        &package.0,
    )
    .unwrap();
    server.join().unwrap();
    let envelope: Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(envelope["value"]["passed"], false);
    assert_eq!(envelope["value"]["count"].as_u64(), Some(9007199254740993));
    assert_eq!(envelope["observed_at"], "2026-01-02T03:04:05Z");
}

#[test]
fn official_sdk_invokes_direct_named_mcp_and_preserves_failures() {
    let package = Directory::new();
    fs::write(
        package.0.join("server.py"),
        include_str!("fixtures/direct_mcp.py"),
    )
    .unwrap();
    let toml = "[tools.read]\nkind='mcp_stdio'\nversion='1'\nargv=['python3','server.py']\ninherit_env=['PATH']\nmax_output_bytes=4096\n[tools.read.direct]\ntool='measurement'\nvalue_pointer='/value'\nobserved_at_pointer='/observed_at'\n";
    let registry = package.manifest(toml);
    let result = acquire(
        &registry.bindings["read"],
        json!({"case":"value"}),
        &package.0,
    )
    .unwrap();
    let envelope: Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(envelope["value"]["passed"], false);
    assert_eq!(envelope["value"]["count"].as_u64(), Some(9007199254740993));
    assert_eq!(envelope["observed_at"], "2026-01-02T03:04:05Z");
    assert!(
        acquire(
            &registry.bindings["read"],
            json!({"case":"error"}),
            &package.0
        )
        .unwrap()
        .error
        .unwrap()
        .message
        .contains("reported acquisition failure")
    );
    assert!(
        acquire(
            &registry.bindings["read"],
            json!({"case":"large"}),
            &package.0
        )
        .unwrap()
        .error
        .unwrap()
        .message
        .contains("output allowance")
    );
    let text = package.manifest(&toml.replace(
        "tool='measurement'",
        "tool='measurement'\nresponse_format='json_text'",
    ));
    assert!(acquire(&text.bindings["read"], json!({"case":"text"}), &package.0).is_ok());
    assert!(
        acquire(
            &text.bindings["read"],
            json!({"case":"duplicate"}),
            &package.0
        )
        .unwrap()
        .error
        .is_some()
    );
    assert!(
        acquire(&registry.bindings["read"], json!({"unknown":1}), &package.0)
            .unwrap_err()
            .message
            .contains("input schema")
    );
}

#[test]
fn pinned_schema_preflight_blocks_wrong_inputs_and_changed_schema_bytes() {
    use sha2::{Digest, Sha256};
    let package = Directory::new();
    let bytes = br#"{"type":"object","properties":{"n":{"type":"integer"}},"required":["n"],"additionalProperties":false}"#;
    fs::write(package.0.join("input.schema.json"), bytes).unwrap();
    let hash = format!("{:x}", Sha256::digest(bytes));
    let registry = package.manifest(&format!("[tools.read]\nkind='http_get'\nversion='1'\n[tools.read.direct]\nurl='http://127.0.0.1:1/data'\n[tools.read.direct.input_schema]\npath='input.schema.json'\nsha256='{hash}'\n"));
    let binding = &registry.bindings["read"];
    binding.preflight(&package.0).unwrap();
    assert!(
        acquire(binding, json!({"n":"wrong"}), &package.0)
            .unwrap_err()
            .message
            .contains("schema")
    );
    fs::write(package.0.join("input.schema.json"), b"{}").unwrap();
    assert!(
        binding
            .preflight(&package.0)
            .unwrap_err()
            .message
            .contains("identity differs")
    );
}

#[test]
fn direct_mcp_http_uses_the_same_sdk_operation_contract() {
    use std::io::{BufRead, BufReader};
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let server = std::thread::spawn(move || {
        for _ in 0..4 {
            let (mut socket, _) = listener.accept().unwrap();
            socket
                .set_read_timeout(Some(std::time::Duration::from_secs(5)))
                .unwrap();
            let mut reader = BufReader::new(socket.try_clone().unwrap());
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
            assert!(line.starts_with("POST /mcp "));
            let mut length = 0;
            loop {
                line.clear();
                reader.read_line(&mut line).unwrap();
                if line == "\r\n" {
                    break;
                }
                if let Some((name, value)) = line.split_once(':')
                    && name.eq_ignore_ascii_case("content-length")
                {
                    length = value.trim().parse::<usize>().unwrap();
                }
            }
            assert!(length < 4096);
            let mut body = vec![0; length];
            reader.read_exact(&mut body).unwrap();
            let message: Value = serde_json::from_slice(&body).unwrap();
            if message["method"] == "notifications/initialized" {
                write!(
                    socket,
                    "HTTP/1.1 202 Accepted\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
                )
                .unwrap();
                continue;
            }
            let result = match message["method"].as_str().unwrap() {
                "initialize" => {
                    json!({"protocolVersion":"2025-11-25","capabilities":{"tools":{}},"serverInfo":{"name":"synthetic-http","version":"1"}})
                }
                "tools/list" => json!({"tools":[{"name":"read","inputSchema":{"type":"object"}}]}),
                "tools/call" => {
                    assert_eq!(message["params"]["arguments"], json!({"site":"lab"}));
                    json!({"structuredContent":{"value":{"passed":false},"observed_at":"2026-01-02T03:04:05Z"},"content":[]})
                }
                other => panic!("unexpected MCP operation {other}"),
            };
            let response =
                serde_json::to_string(&json!({"jsonrpc":"2.0","id":message["id"],"result":result}))
                    .unwrap();
            write!(socket,"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{response}",response.len()).unwrap();
        }
    });
    let package = Directory::new();
    let registry=package.manifest(&format!("[tools.read]\nkind='mcp_http'\nversion='1'\ntimeout_seconds=5\n[tools.read.direct]\nurl='http://{address}/mcp'\ntool='read'\nvalue_pointer='/value'\nobserved_at_pointer='/observed_at'\n"));
    let result = acquire(
        &registry.bindings["read"],
        json!({"site":"lab"}),
        &package.0,
    )
    .unwrap();
    server.join().unwrap();
    let envelope: Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(envelope["value"]["passed"], false);
    assert_eq!(envelope["observed_at"], "2026-01-02T03:04:05Z");
}

#[path = "common/companion.rs"]
mod companion;
