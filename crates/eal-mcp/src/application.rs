//! Validate operator policy, construct one service and select the MCP transport.
use crate::{
    auth::HttpPolicy,
    server::{Server, ServerOptions},
    settings::ServerSettings,
    transport,
};
use eal_runtime::bootstrap::{RuntimeOptions, build_service};
use std::{collections::BTreeMap, sync::Arc};
use tokio_util::sync::CancellationToken;

pub struct Application {
    pub server: Server,
    pub settings: ServerSettings,
    pub http_policy: HttpPolicy,
}
pub fn build(
    settings: ServerSettings,
    environ: &BTreeMap<String, String>,
) -> Result<Application, String> {
    settings.validate()?;
    let http_policy = HttpPolicy::from_settings(&settings, environ)?;
    let mut options = RuntimeOptions::new(settings.workspace.clone());
    options.registry = settings.registry.clone();
    options.database = settings.database.clone();
    options.limits = settings.limits.clone();
    options.methods = settings.methods.clone();
    let service = Arc::new(build_service(options)?);
    let server = Server::new(
        service,
        ServerOptions {
            known_entries: settings.known_entries.clone(),
            exposure: Some(settings.exposure.clone()),
            max_in_flight: settings.max_in_flight,
        },
    )?;
    Ok(Application {
        server,
        settings,
        http_policy,
    })
}
pub async fn run(application: Application) -> Result<(), String> {
    if application.settings.transport == "stdio" {
        return transport::serve_stdio(application.server).await;
    }
    let listener = tokio::net::TcpListener::bind((
        application.settings.host.as_str(),
        application.settings.port,
    ))
    .await
    .map_err(|e| e.to_string())?;
    let cancellation = CancellationToken::new();
    let signal = cancellation.clone();
    let signal_task = tokio::spawn(async move {
        if tokio::signal::ctrl_c().await.is_ok() {
            signal.cancel();
        }
    });
    let result = transport::serve_http(
        application.server,
        listener,
        &application.settings.path,
        application.http_policy,
        cancellation,
    )
    .await;
    signal_task.abort();
    result
}

pub fn environment() -> Result<BTreeMap<String, String>, String> {
    std::env::vars_os()
        .map(|(key, value)| {
            Ok((
                key.into_string()
                    .map_err(|_| "Environment names must be UTF-8")?,
                value
                    .into_string()
                    .map_err(|_| "Environment values must be UTF-8")?,
            ))
        })
        .collect()
}
