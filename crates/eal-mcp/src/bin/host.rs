#[tokio::main]
async fn main() -> std::process::ExitCode {
    eal_mcp::host::main().await
}
