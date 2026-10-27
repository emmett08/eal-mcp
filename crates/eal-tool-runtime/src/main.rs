//! Standalone isolated package launcher; stdout contains only the observation.
#![forbid(unsafe_code)]
use eal_tool_runtime::{Error, package::ToolPackage};
use std::{
    io::{Read, Write},
    path::PathBuf,
};
struct CancelOnDrop(eal_process::CancellationToken);
impl Drop for CancelOnDrop {
    fn drop(&mut self) {
        self.0.cancel();
    }
}
async fn termination() -> Result<(), Error> {
    #[cfg(unix)]
    {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?
            .recv()
            .await;
    }
    #[cfg(not(unix))]
    {
        std::future::pending::<()>().await;
    }
    Ok(())
}
async fn run() -> Result<(), Error> {
    let args = std::env::args_os().skip(1).collect::<Vec<_>>();
    if args.len() != 2 || args[0] != "--package" {
        return Err(Error::InvalidInput(
            "usage: eal-tool-run --package tool.toml".into(),
        ));
    }
    let package = ToolPackage::load(&PathBuf::from(&args[1]))?;
    let mut bytes = Vec::new();
    std::io::stdin().take(1_048_577).read_to_end(&mut bytes)?;
    if bytes.len() > 1_048_576 {
        return Err(Error::Budget("tool request exceeds 1 MiB".into()));
    }
    let current = std::env::current_exe()?;
    let supervisor =
        eal_process::companion_in(current.parent().ok_or_else(|| {
            Error::InvalidInput("tool runner has no installation directory".into())
        })?);
    let cancellation = eal_process::CancellationToken::default();
    let _guard = CancelOnDrop(cancellation.clone());
    let worker_token = cancellation.clone();
    let mut worker =
        tokio::task::spawn_blocking(move || package.execute(&bytes, &supervisor, &worker_token));
    let result = tokio::select! {
        result=&mut worker=>result,
        signal=tokio::signal::ctrl_c()=> { cancellation.cancel();let _=worker.await;signal?;return Err(Error::ToolExecution("tool interrupted".into())); }
        signal=termination()=> { cancellation.cancel();let _=worker.await;signal?;return Err(Error::ToolExecution("tool terminated".into())); }
    };
    let value = result.map_err(|_| Error::ToolExecution("tool worker failed".into()))??;
    let mut output = std::io::stdout().lock();
    serde_json::to_writer(&mut output, &value)?;
    output.write_all(b"\n")?;
    Ok(())
}
#[tokio::main]
async fn main() {
    if let Err(error) = run().await {
        eprintln!("eal-tool-run: {error}");
        std::process::exit(1);
    }
}
