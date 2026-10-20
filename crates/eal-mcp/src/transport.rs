//! Official SDK stdio and stateless Streamable HTTP transports.
use crate::{auth::HttpPolicy, server::Server};
use axum::{
    Router,
    extract::{Request, State},
    middleware::{self, Next},
    response::{IntoResponse, Response},
};
use http::{HeaderMap, StatusCode};
use rmcp::{
    RoleServer, ServiceExt,
    transport::{
        async_rw::AsyncRwTransport,
        streamable_http_server::{
            StreamableHttpServerConfig, StreamableHttpService, session::never::NeverSessionManager,
        },
    },
};
use std::sync::Arc;
use tokio_util::sync::CancellationToken;

pub const MAX_MESSAGE_BYTES: usize = 4 * 1024 * 1024;

pub async fn serve_stdio(server: Server) -> Result<(), String> {
    let (reader, writer) = rmcp::transport::io::stdio_pipes();
    let transport = AsyncRwTransport::<RoleServer, _, _>::new(reader, writer)
        .with_max_line_length(MAX_MESSAGE_BYTES);
    server
        .serve(transport)
        .await
        .map_err(|error| error.to_string())?
        .waiting()
        .await
        .map_err(|error| error.to_string())?;
    Ok(())
}

pub fn http_router(
    server: Server,
    path: &str,
    policy: HttpPolicy,
    cancellation: CancellationToken,
) -> Result<Router, String> {
    if !path.starts_with('/')
        || path.starts_with("//")
        || path.chars().any(|character| {
            character.is_whitespace() || character.is_control() || "?#\\{}".contains(character)
        })
        || path.split('/').any(|part| matches!(part, "." | ".."))
    {
        return Err(
            "path must be an absolute HTTP path without query, fragment or traversal".into(),
        );
    }
    let config = StreamableHttpServerConfig::default()
        .with_legacy_session_mode(false)
        .with_json_response(true)
        .with_stateless_protocol_metadata_required(false)
        .with_max_request_body_bytes(MAX_MESSAGE_BYTES)
        .with_cancellation_token(cancellation)
        .disable_allowed_hosts()
        .disable_allowed_origins();
    let service = StreamableHttpService::new(
        move || Ok(server.clone()),
        Arc::new(NeverSessionManager::default()),
        config,
    );
    Ok(Router::new()
        .without_v07_checks()
        .route_service(path, service)
        .layer(middleware::from_fn_with_state(policy, authorize)))
}

pub async fn serve_http(
    server: Server,
    listener: tokio::net::TcpListener,
    path: &str,
    policy: HttpPolicy,
    cancellation: CancellationToken,
) -> Result<(), String> {
    let router = http_router(server, path, policy, cancellation.clone())?;
    axum::serve(listener, router)
        .with_graceful_shutdown(cancellation.cancelled_owned())
        .await
        .map_err(|error| error.to_string())
}

fn one_header<'a>(headers: &'a HeaderMap, name: &str) -> Result<Option<&'a str>, ()> {
    let mut values = headers.get_all(name).iter();
    let Some(value) = values.next() else {
        return Ok(None);
    };
    if values.next().is_some() {
        return Err(());
    }
    value.to_str().map(Some).map_err(|_| ())
}
async fn authorize(State(policy): State<HttpPolicy>, request: Request, next: Next) -> Response {
    let headers = request.headers();
    let values = (
        one_header(headers, "host"),
        one_header(headers, "origin"),
        one_header(headers, "authorization"),
    );
    let (Ok(host), Ok(origin), Ok(authorization)) = values else {
        return (
            StatusCode::BAD_REQUEST,
            "Ambiguous or invalid HTTP admission headers",
        )
            .into_response();
    };
    if let Err(error) = policy.authorize(host, origin, authorization) {
        return (
            StatusCode::from_u16(error.status).unwrap_or(StatusCode::FORBIDDEN),
            error.message,
        )
            .into_response();
    }
    match one_header(headers, "mcp-protocol-version") {
        Err(()) => {
            return (
                StatusCode::BAD_REQUEST,
                "Ambiguous or invalid MCP protocol header",
            )
                .into_response();
        }
        Ok(Some(version))
            if !rmcp::model::ProtocolVersion::known_up_to(
                &rmcp::model::ProtocolVersion::V_2025_11_25,
            )
            .iter()
            .any(|known| known.as_str() == version) =>
        {
            return (StatusCode::BAD_REQUEST, "Unsupported MCP protocol version").into_response();
        }
        _ => {}
    }
    next.run(request).await
}
