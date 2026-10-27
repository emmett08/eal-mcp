#![cfg(unix)]
//! MCP sessions use the production supervisor and preserve the acquisition lease.
use eal_runtime::{
    acquisition::{self, ToolRegistry},
    lease::WorkspaceAcquisition,
    process::Supervisor,
};
use serde_json::json;
use std::{
    fs,
    path::PathBuf,
    process::{Command, Stdio},
    time::{Duration, Instant},
};
#[path = "common/companion.rs"]
mod companion;

const SERVER: &str = r#"import json,sys,subprocess,time,pathlib
for line in sys.stdin:
    message=json.loads(line)
    method=message.get('method')
    if method=='initialize': result={'protocolVersion':'2025-11-25','capabilities':{'tools':{}},'serverInfo':{'name':'lifecycle-fixture','version':'1'}}
    elif method=='notifications/initialized': continue
    elif method=='tools/list': result={'tools':[{'name':'read','inputSchema':{'type':'object'}}]}
    elif method=='tools/call':
        child=subprocess.Popen(['/bin/sleep','30'])
        pathlib.Path('child.pid').write_text(str(child.pid))
        if message['params']['arguments']['wait']: time.sleep(30)
        result={'content':[],'structuredContent':{'value':{'passed':True}}}
    else: continue
    print(json.dumps({'jsonrpc':'2.0','id':message['id'],'result':result}),flush=True)
"#;
struct Directory(PathBuf);
impl Directory {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!("eal-mcp-lifetime-{}", uuid::Uuid::new_v4()));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn registry(&self, timeout: f64) -> ToolRegistry {
        fs::write(self.0.join("server.py"), SERVER).unwrap();
        fs::write(self.0.join("tools.toml"),format!("[tools.read]\nkind='mcp_stdio'\nversion='1'\nargv=['python3','server.py']\ninherit_env=['PATH']\ntimeout_seconds={timeout}\nmax_output_bytes=8192\n[tools.read.direct]\ntool='read'\nvalue_pointer='/value'\n")).unwrap();
        ToolRegistry::load(self.0.join("tools.toml")).unwrap()
    }
    fn child(&self) -> rustix::process::Pid {
        rustix::process::Pid::from_raw(
            fs::read_to_string(self.0.join("child.pid"))
                .unwrap()
                .parse()
                .unwrap(),
        )
        .unwrap()
    }
}
impl Drop for Directory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn stopped(child: rustix::process::Pid) {
    assert_eq!(
        rustix::process::test_kill_process(child),
        Err(rustix::io::Errno::SRCH)
    );
}
fn run(directory: &Directory, timeout: f64, wait: bool) -> bool {
    let registry = directory.registry(timeout);
    let acquisition = WorkspaceAcquisition::new(&directory.0).unwrap();
    let lease = acquisition.hold(Duration::from_secs(1)).unwrap();
    let result = acquisition::acquire(
        &registry.bindings["read"],
        &json!({"input":{"wait":wait}}),
        &directory.0,
        &[17; 32],
        &Supervisor::new(companion::path()),
        Some(&lease),
    );
    stopped(directory.child());
    drop(lease);
    // The companion releases its inherited descriptor only after group cleanup.
    let _again = acquisition.hold(Duration::from_millis(250)).unwrap();
    result.is_ok_and(|r| r.error.is_none())
}
#[test]
fn completed_mcp_session_reaps_unreturned_children_before_releasing_its_lease() {
    assert!(run(&Directory::new(), 3.0, false));
}
#[test]
fn timed_out_mcp_session_stops_children_and_releases_the_lease() {
    let started = Instant::now();
    assert!(!run(&Directory::new(), 0.5, true));
    assert!(started.elapsed() < Duration::from_secs(2));
}
#[test]
fn abrupt_mcp_host_death_keeps_lease_until_child_cleanup() {
    const KEY: &str = "EAL_MCP_HOST_DEATH_FIXTURE";
    if let Some(path) = std::env::var_os(KEY) {
        let directory = Directory(PathBuf::from(path));
        let _ = run(&directory, 10.0, true);
        panic!("Fixture owner should have been killed during acquisition");
    }
    let directory = Directory::new();
    let mut owner = Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "abrupt_mcp_host_death_keeps_lease_until_child_cleanup",
            "--nocapture",
        ])
        .env(KEY, &directory.0)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let started = Instant::now();
    while !directory.0.join("child.pid").exists() {
        assert!(started.elapsed() < Duration::from_secs(4));
        assert!(owner.try_wait().unwrap().is_none());
        std::thread::sleep(Duration::from_millis(5));
    }
    let child = directory.child();
    owner.kill().unwrap();
    owner.wait().unwrap();
    let acquisition = WorkspaceAcquisition::new(&directory.0).unwrap();
    let _lease = acquisition.hold(Duration::from_secs(2)).unwrap();
    stopped(child);
}
