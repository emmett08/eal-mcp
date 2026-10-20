use eal_coordination::{
    lab_server::{LabServer, OPERATION_NAMES},
    lab_store::LabStore,
};
use rmcp::{ServiceExt, model::CallToolRequestParams, transport::TokioChildProcess};
use serde_json::{Value, json};
use std::path::PathBuf;
struct Sandbox(PathBuf);
impl Sandbox {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!("eal-lab-server-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Sandbox {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn exact_ten_tools_keep_strict_integer_admission_before_rows() {
    let sandbox = Sandbox::new();
    let server = LabServer::new(LabStore::new(sandbox.0.join("lab.sqlite")).unwrap()).unwrap();
    let mut names = server
        .tools()
        .iter()
        .map(|t| t.name.as_ref())
        .collect::<Vec<_>>();
    names.sort();
    let mut wanted = OPERATION_NAMES.to_vec();
    wanted.sort();
    assert_eq!(names, wanted);
    for value in [
        json!(true),
        json!(1.0),
        json!("1"),
        json!({"$serde_json::private::Number":"1"}),
    ] {
        let result = server.call(
            "mas_create_run",
            &json!({"run_id":"r","agents":["a","b"],"max_calls":value}),
        );
        assert_eq!(result["ok"], false);
        assert_eq!(result["error"]["code"], "invalid_input");
    }
    assert_eq!(
        server.call("mas_status", &json!({"run_id":"r"}))["error"]["message"],
        "unknown run"
    );
    assert_eq!(
        server.call("mas_create_run", &json!({"run_id":"r","agents":["a","b"]}))["ok"],
        true
    );
    assert_eq!(
        server.call(
            "mas_receive",
            &json!({"run_id":"r","recipient":"b","limit":false})
        )["error"]["code"],
        "invalid_input"
    );
}

#[test]
fn in_process_codec_retains_exact_values_and_reports_truthful_metrics() {
    let sandbox = Sandbox::new();
    let server = LabServer::new(LabStore::new(sandbox.0.join("lab.sqlite")).unwrap()).unwrap();
    let envelope = json!({"message_id":"m","run_id":"r","sender":"a","recipient":"b","version":"1","kind":"request","payload":{"$serde_json::private::Number":"ordinary","$eal.literal-key/private":{"n":123456789012345678901234567890u128},"ok":false,"zero":0}});
    let encoded = server.call("mas_encode", &json!({"envelope":envelope}));
    assert_eq!(encoded["ok"], true);
    let decoded = server.call("mas_validate", &json!({"text":encoded["result"]}));
    assert_eq!(decoded["result"], envelope);
    let invalid = server.call("mas_encode", &json!({"envelope":{"bad":true}}));
    assert_eq!(invalid["ok"], false);
    let info = server.call("mas_runtime_info", &json!({}));
    assert_eq!(info["result"]["codec_mode"], "in_process");
    assert!(info["result"]["codec_pid"].is_null());
    assert_eq!(info["result"]["codec_starts"], 0);
    assert_eq!(info["result"]["parse_calls"], 1);
    assert_eq!(info["result"]["encode_calls"], 2);
    let _: Value = info;
}

#[tokio::test]
async fn actual_sdk_stdio_keeps_codec_lazy_and_mailbox_rejections_recoverable() {
    let sandbox = Sandbox::new();
    let database = sandbox.0.join("stdio.sqlite");
    let mut command = tokio::process::Command::new(env!("CARGO_BIN_EXE_eal-mas-server"));
    command.arg("--database").arg(&database).kill_on_drop(true);
    let client = ().serve(TokioChildProcess::new(command).unwrap()).await.unwrap();
    assert!(!database.exists());
    assert_eq!(client.list_tools(None).await.unwrap().tools.len(), 10);
    assert!(!database.exists());
    let call = |name: &str, arguments: Value| {
        CallToolRequestParams::new(name.to_owned())
            .with_arguments(arguments.as_object().unwrap().clone())
    };
    let envelope = json!({"message_id":"m","run_id":"r","sender":"a","recipient":"b","version":"1","kind":"request","payload":{"$serde_json::private::Number":"ordinary","$eal.literal-key/test":false,"big":123456789012345678901234567890u128}});
    let mut invalid = envelope.clone();
    invalid["payload"]["fraction"] = json!(1.0);
    let result = client
        .call_tool(call(
            "mas_send_envelope",
            json!({"envelope":invalid,"actor":"a"}),
        ))
        .await
        .unwrap();
    assert_ne!(result.is_error, Some(true));
    assert_eq!(result.structured_content.unwrap()["ok"], false);
    assert!(!database.exists());
    let encoded = client
        .call_tool(call("mas_encode", json!({"envelope":envelope})))
        .await
        .unwrap()
        .structured_content
        .unwrap();
    assert_eq!(encoded["ok"], true);
    assert!(!database.exists());
    let info = client
        .call_tool(call("mas_runtime_info", json!({})))
        .await
        .unwrap()
        .structured_content
        .unwrap();
    assert_eq!(info["result"]["codec_mode"], "in_process");
    assert!(!database.exists());
    let created = client
        .call_tool(call(
            "mas_create_run",
            json!({"run_id":"r","agents":["a","b"]}),
        ))
        .await
        .unwrap()
        .structured_content
        .unwrap();
    assert_eq!(created["ok"], true);
    assert!(database.exists());
    let sent = client
        .call_tool(call(
            "mas_send_envelope",
            json!({"envelope":envelope,"actor":"a"}),
        ))
        .await
        .unwrap()
        .structured_content
        .unwrap();
    assert_eq!(sent["result"]["duplicate"], false);
    let duplicate = client
        .call_tool(call(
            "mas_send_envelope",
            json!({"envelope":envelope,"actor":"a"}),
        ))
        .await
        .unwrap()
        .structured_content
        .unwrap();
    assert_eq!(duplicate["result"]["duplicate"], true);
    let early_ack = client
        .call_tool(call(
            "mas_ack",
            json!({"run_id":"r","message_id":"m","recipient":"b"}),
        ))
        .await
        .unwrap()
        .structured_content
        .unwrap();
    assert_eq!(
        early_ack["error"]["message"],
        "message has not been received"
    );
    let received = client
        .call_tool(call("mas_receive", json!({"run_id":"r","recipient":"b"})))
        .await
        .unwrap()
        .structured_content
        .unwrap();
    assert_eq!(received["result"][0]["message"], envelope);
    assert_eq!(received["result"][0]["redelivered"], false);
    let acknowledged = client
        .call_tool(call(
            "mas_ack",
            json!({"run_id":"r","message_id":"m","recipient":"b"}),
        ))
        .await
        .unwrap()
        .structured_content
        .unwrap();
    assert_eq!(acknowledged["ok"], true);
    let reserved = client
        .call_tool(call(
            "mas_reserve_call",
            json!({"run_id":"r","actor":"a","call_id":"call"}),
        ))
        .await
        .unwrap()
        .structured_content
        .unwrap();
    assert_eq!(reserved["ok"], true);
    let evidence = json!({"execution_id":"software-test","observed":false,"count":0,"ratio":1.0,"$serde_json::private::Number":"ordinary"});
    let finished = client
        .call_tool(call(
            "mas_finish_call",
            json!({"run_id":"r","actor":"a","call_id":"call","evidence":evidence}),
        ))
        .await
        .unwrap()
        .structured_content
        .unwrap();
    assert_eq!(finished["ok"], true);
    let repeated = client
        .call_tool(call(
            "mas_finish_call",
            json!({"run_id":"r","actor":"a","call_id":"call","evidence":evidence}),
        ))
        .await
        .unwrap()
        .structured_content
        .unwrap();
    assert_eq!(repeated["error"]["message"], "call already completed");
    let status = client
        .call_tool(call("mas_status", json!({"run_id":"r"})))
        .await
        .unwrap()
        .structured_content
        .unwrap();
    assert_eq!(status["result"]["calls"][0]["evidence"], evidence);
    assert_eq!(status["result"]["usage"]["messages"], 1);
    let final_info = client
        .call_tool(call("mas_runtime_info", json!({})))
        .await
        .unwrap()
        .structured_content
        .unwrap();
    assert_eq!(
        final_info["result"]["server_pid"],
        info["result"]["server_pid"]
    );
    assert_eq!(final_info["result"]["codec_starts"], 0);
    client.cancel().await.unwrap();
}
