//! Native MAS Protocol Lab SDK server; stdout contains only MCP frames.
use eal_coordination::lab_server::LabServer;
use rmcp::ServiceExt;

#[tokio::main]
async fn main() -> std::process::ExitCode {
    let result = async {
        let arguments = std::env::args_os().skip(1).collect::<Vec<_>>();
        if arguments.len() == 1 && (arguments[0] == "--help" || arguments[0] == "-h") {
            println!("usage: eal-mas-server --database ABSOLUTE_SQLITE_PATH");
            return Ok(());
        }
        if arguments.len() != 2 || arguments[0] != "--database" {
            return Err("usage: eal-mas-server --database ABSOLUTE_SQLITE_PATH".to_owned());
        }
        let database = std::path::PathBuf::from(&arguments[1]);
        if !database.is_absolute() {
            return Err("The state database path must be absolute.".into());
        }
        let server = LabServer::for_database(database)?;
        let running = server
            .serve(rmcp::transport::stdio())
            .await
            .map_err(|e| format!("MAS stdio initialization failed: {e}"))?;
        running
            .waiting()
            .await
            .map_err(|e| format!("MAS stdio service failed: {e}"))?;
        Ok::<_, String>(())
    }
    .await;
    match result {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            std::process::ExitCode::from(2)
        }
    }
}
