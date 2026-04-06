//! HTTP API for JSON → CCSDS → UDP (optional static UI).

use std::sync::Arc;

use axum::body::Bytes;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::Json;
use axum::Router;
use serde::Serialize;
use tokio::sync::Mutex;
use tower_http::cors::CorsLayer;
use tower_http::services::{ServeDir, ServeFile};

use crate::{command_dictionary_entries, CcsdsPacket, CommandMetadata, SpaceCommand, UdpSender};

/// Shared HTTP state (UDP sender to CI_LAB).
pub struct AppState {
    pub udp: Mutex<UdpSender>,
}

#[derive(Debug, Serialize)]
pub struct SendResponse {
    pub bytes_sent: usize,
    pub wire_length: usize,
}

#[derive(Debug)]
pub enum ApiError {
    BadRequest(String),
    Io(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, msg) = match self {
            ApiError::BadRequest(m) => (StatusCode::BAD_REQUEST, m),
            ApiError::Io(m) => (StatusCode::INTERNAL_SERVER_ERROR, m),
        };
        let body = Json(serde_json::json!({ "error": msg }));
        (status, body).into_response()
    }
}

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "ok" }))
}

async fn commands() -> Json<Vec<CommandMetadata>> {
    Json(command_dictionary_entries())
}

async fn send_json(
    State(state): State<Arc<AppState>>,
    body: Bytes,
) -> Result<Json<SendResponse>, ApiError> {
    let s = String::from_utf8(body.to_vec()).map_err(|e| ApiError::BadRequest(e.to_string()))?;
    let cmd = SpaceCommand::from_json(s.trim()).map_err(|e| ApiError::BadRequest(e.to_string()))?;
    let packet =
        CcsdsPacket::from_command(&cmd).map_err(|e| ApiError::BadRequest(e.to_string()))?;
    let wire_len = 6 + packet.payload.len() + 2;
    let guard = state.udp.lock().await;
    let n = guard
        .send_packet(&packet)
        .map_err(|e| ApiError::Io(e.to_string()))?;
    Ok(Json(SendResponse {
        bytes_sent: n,
        wire_length: wire_len,
    }))
}

/// Builds the API router (nested under `/api` by [`build_app`]).
pub fn api_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/commands", get(commands))
        .route("/send", post(send_json))
        .with_state(state)
}

/// Full application: `/api/*` JSON API, optional SPA fallback from `static_dir`.
pub fn build_app(sender: UdpSender, static_dir: Option<&str>) -> Router {
    let state = Arc::new(AppState {
        udp: Mutex::new(sender),
    });
    let api = api_router(state.clone());

    let mut app = Router::new()
        .nest("/api", api)
        .route("/health", get(health))
        .layer(CorsLayer::permissive());

    if let Some(root) = static_dir {
        let index = format!("{}/index.html", root.trim_end_matches('/'));
        let static_svc = ServeDir::new(root).not_found_service(ServeFile::new(index));
        app = app.fallback_service(static_svc);
    }
    app
}

/// Runs the HTTP server until SIGINT (Ctrl+C).
pub async fn run() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let bind = std::env::var("BRIDGE_HTTP_BIND").unwrap_or_else(|_| "127.0.0.1:8080".to_string());
    let udp_target =
        std::env::var("BRIDGE_UDP_TARGET").unwrap_or_else(|_| "127.0.0.1:1234".to_string());
    let static_dir = std::env::var("BRIDGE_STATIC_DIR").ok();

    let sender = UdpSender::connect(&udp_target)?;
    let app = build_app(sender, static_dir.as_deref());

    let listener = tokio::net::TcpListener::bind(&bind).await?;
    eprintln!("bridge-server: listening on http://{bind} (UDP to {udp_target})");
    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use http_body_util::BodyExt;
    use std::net::UdpSocket;
    use std::thread;
    use std::time::Duration;
    use tower::ServiceExt;

    #[tokio::test]
    async fn api_commands_returns_dictionary() {
        let recv = UdpSocket::bind("127.0.0.1:0").expect("bind");
        let addr = recv.local_addr().expect("addr");
        let sender = UdpSender::connect(&addr.to_string()).expect("connect");
        let app = build_app(sender, None);

        let res = app
            .oneshot(
                Request::builder()
                    .uri("/api/commands")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("oneshot");

        assert_eq!(res.status(), StatusCode::OK);
        let body = res.into_body().collect().await.unwrap().to_bytes();
        let v: serde_json::Value = serde_json::from_slice(&body).expect("json");
        let arr = v.as_array().expect("array");
        assert!(!arr.is_empty());
        assert_eq!(arr[0]["name"], "CMD_HEARTBEAT");
    }

    #[tokio::test]
    async fn api_send_round_trip_udp() {
        let recv = UdpSocket::bind("127.0.0.1:0").expect("bind");
        recv.set_read_timeout(Some(Duration::from_secs(2)))
            .expect("timeout");
        let addr = recv.local_addr().expect("addr");

        let handle = thread::spawn(move || {
            let mut buf = [0u8; 2048];
            let n = recv.recv(&mut buf).expect("recv");
            buf[..n].to_vec()
        });

        thread::sleep(Duration::from_millis(20));

        let sender = UdpSender::connect(&addr.to_string()).expect("connect");
        let app = build_app(sender, None);

        let body = r#"{"command":"CMD_HEARTBEAT","sequence_count":0}"#;
        let res = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/send")
                    .header("content-type", "application/json")
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .expect("oneshot");

        assert_eq!(res.status(), StatusCode::OK);
        let wire = handle.join().expect("thread");
        assert!(!wire.is_empty());
        assert_eq!(wire.len(), 11);
    }

    #[tokio::test]
    async fn api_send_rejects_bad_json() {
        let recv = UdpSocket::bind("127.0.0.1:0").expect("bind");
        let addr = recv.local_addr().expect("addr");
        let sender = UdpSender::connect(&addr.to_string()).expect("connect");
        let app = build_app(sender, None);

        let res = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/send")
                    .header("content-type", "application/json")
                    .body(Body::from("not json"))
                    .unwrap(),
            )
            .await
            .expect("oneshot");

        assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    }
}
