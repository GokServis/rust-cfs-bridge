//! HTTP API for JSON → CCSDS → UDP (optional static UI).

use std::sync::Arc;

use axum::body::Bytes;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, get_service, post};
use axum::Json;
use axum::Router;
use serde::Serialize;
use tokio::sync::broadcast;
use tokio::sync::Mutex;
use tower_http::cors::CorsLayer;
use tower_http::services::{ServeDir, ServeFile};

use crate::tlm::TlmEvent;
use crate::{
    command_dictionary_entries, command_dictionary_resolve, CcsdsPacket, CommandMetadata,
    SpaceCommand, UdpSender,
};

/// Shared HTTP state (UDP sender to CI_LAB).
pub struct AppState {
    pub udp: Mutex<UdpSender>,
    pub tlm_tx: broadcast::Sender<TlmEvent>,
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
    /// CI_LAB / UDP uplink target not reachable (e.g. cFS not running).
    UpstreamUnavailable(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, msg) = match self {
            ApiError::BadRequest(m) => (StatusCode::BAD_REQUEST, m),
            ApiError::Io(m) => (StatusCode::INTERNAL_SERVER_ERROR, m),
            ApiError::UpstreamUnavailable(m) => (StatusCode::SERVICE_UNAVAILABLE, m),
        };
        let body = Json(serde_json::json!({ "error": msg }));
        (status, body).into_response()
    }
}

fn map_udp_send_err(e: std::io::Error) -> ApiError {
    let msg = e.to_string();
    if e.kind() == std::io::ErrorKind::ConnectionRefused || msg.contains("Connection refused") {
        let target = std::env::var("BRIDGE_UDP_TARGET").unwrap_or_else(|_| "127.0.0.1:1234".to_string());
        return ApiError::UpstreamUnavailable(format!(
            "UDP target not reachable ({target}): CI_LAB is not listening. Start cFS with: docker compose --profile cfs up --build (or make up-cfs), or run a UDP sink on that port for testing."
        ));
    }
    ApiError::Io(msg)
}

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "ok" }))
}

async fn commands() -> Json<Vec<CommandMetadata>> {
    Json(command_dictionary_entries())
}

async fn telemetry_ws(
    State(state): State<Arc<AppState>>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_telemetry_ws(socket, state))
}

async fn handle_telemetry_ws(mut socket: WebSocket, state: Arc<AppState>) {
    let mut rx = state.tlm_tx.subscribe();
    loop {
        match rx.recv().await {
            Ok(ev) => {
                let Ok(json) = serde_json::to_string(&ev) else {
                    continue;
                };
                if socket.send(Message::Text(json)).await.is_err() {
                    break;
                }
            }
            Err(broadcast::error::RecvError::Lagged(_)) => continue,
            Err(broadcast::error::RecvError::Closed) => break,
        }
    }
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
        .map_err(map_udp_send_err)?;
    Ok(Json(SendResponse {
        bytes_sent: n,
        wire_length: wire_len,
    }))
}

async fn to_lab_output_enable(
    State(state): State<Arc<AppState>>,
) -> Result<Json<SendResponse>, ApiError> {
    let cmd = command_dictionary_resolve("CMD_TO_LAB_ENABLE_OUTPUT", 0, None)
        .map_err(|e| ApiError::BadRequest(e.to_string()))?;
    let packet =
        CcsdsPacket::from_command(&cmd).map_err(|e| ApiError::BadRequest(e.to_string()))?;
    let wire_len = 6 + packet.payload.len() + 2;
    let guard = state.udp.lock().await;
    let n = guard
        .send_packet(&packet)
        .map_err(map_udp_send_err)?;
    Ok(Json(SendResponse {
        bytes_sent: n,
        wire_length: wire_len,
    }))
}

async fn to_lab_output_disable(
    State(state): State<Arc<AppState>>,
) -> Result<Json<SendResponse>, ApiError> {
    let cmd = command_dictionary_resolve("CMD_TO_LAB_DISABLE_OUTPUT", 0, None)
        .map_err(|e| ApiError::BadRequest(e.to_string()))?;
    let packet =
        CcsdsPacket::from_command(&cmd).map_err(|e| ApiError::BadRequest(e.to_string()))?;
    let wire_len = 6 + packet.payload.len() + 2;
    let guard = state.udp.lock().await;
    let n = guard
        .send_packet(&packet)
        .map_err(map_udp_send_err)?;
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
        .route("/to_lab/output/enable", post(to_lab_output_enable))
        .route("/to_lab/output/disable", post(to_lab_output_disable))
        .route("/tlm/ws", get(telemetry_ws))
        .with_state(state)
}

/// Full application: `/api/*` JSON API, optional SPA fallback from `static_dir`.
pub fn build_app(
    sender: UdpSender,
    tlm_tx: broadcast::Sender<TlmEvent>,
    static_dir: Option<&str>,
) -> Router {
    let state = Arc::new(AppState {
        udp: Mutex::new(sender),
        tlm_tx,
    });
    let api = api_router(state.clone());

    let mut app = Router::new()
        .nest("/api", api)
        .route("/health", get(health))
        .layer(CorsLayer::permissive());

    if let Some(root) = static_dir {
        let index = format!("{}/index.html", root.trim_end_matches('/'));
        let static_svc = ServeDir::new(root).not_found_service(ServeFile::new(index.clone()));
        /* Explicit HTML routes: `fallback_service` + `ServeDir` alone did not serve `index.html`
         * for `/` or `/telemetry` in this Axum/tower-http combination. */
        app = app
            .route_service("/", get_service(ServeFile::new(index.clone())))
            .route_service("/telemetry", get_service(ServeFile::new(index.clone())))
            .fallback_service(static_svc);
    }
    app
}

/// Runs the HTTP server until SIGINT (Ctrl+C).
pub async fn run() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let bind = std::env::var("BRIDGE_HTTP_BIND").unwrap_or_else(|_| "127.0.0.1:8080".to_string());
    let udp_target =
        std::env::var("BRIDGE_UDP_TARGET").unwrap_or_else(|_| "127.0.0.1:1234".to_string());
    let tlm_bind = std::env::var("BRIDGE_TLM_BIND")
        .unwrap_or_else(|_| crate::BRIDGE_TLM_DEFAULT_BIND.to_string());
    let static_dir = std::env::var("BRIDGE_STATIC_DIR").ok();

    let sender = UdpSender::connect(&udp_target)?;
    let (tlm_tx, _tlm_rx) = broadcast::channel::<TlmEvent>(256);
    let tlm_addr: std::net::SocketAddr = tlm_bind.parse()?;
    tokio::spawn(crate::tlm::udp_task::run_udp_telemetry_listener_supervised(
        tlm_addr,
        tlm_tx.clone(),
    ));
    let app = build_app(sender, tlm_tx, static_dir.as_deref());

    let listener = tokio::net::TcpListener::bind(&bind).await?;
    eprintln!("bridge-server: listening on http://{bind} (UDP to {udp_target}, telemetry UDP on {tlm_bind})");
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
        let (tlm_tx, _) = broadcast::channel(4);
        let app = build_app(sender, tlm_tx, None);

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
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0]["name"], "CMD_HEARTBEAT");
        assert!(arr.iter().any(|e| e["name"] == "CMD_PING"));
        assert!(!arr.iter().any(|e| e["name"] == "CMD_TO_LAB_ENABLE_OUTPUT"));
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
        let (tlm_tx, _) = broadcast::channel(4);
        let app = build_app(sender, tlm_tx, None);

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
    async fn api_to_lab_output_enable_sends_udp() {
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
        let (tlm_tx, _) = broadcast::channel(4);
        let app = build_app(sender, tlm_tx, None);

        let res = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/to_lab/output/enable")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("oneshot");
        assert_eq!(res.status(), StatusCode::OK);

        let wire = handle.join().expect("thread");
        // 6-byte header + 16-byte payload + 2-byte CRC
        assert_eq!(wire.len(), 24);
    }

    #[tokio::test]
    async fn api_to_lab_output_disable_sends_udp() {
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
        let (tlm_tx, _) = broadcast::channel(4);
        let app = build_app(sender, tlm_tx, None);

        let res = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/to_lab/output/disable")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("oneshot");
        assert_eq!(res.status(), StatusCode::OK);

        let wire = handle.join().expect("thread");
        // 6-byte header + 0-byte payload + 2-byte CRC
        assert_eq!(wire.len(), 8);
    }

    #[tokio::test]
    async fn get_telemetry_serves_spa_index_when_static_dir_set() {
        let dir = std::env::temp_dir().join(format!("rb_spa_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("index.html"),
            "<!doctype html><html><body>spa</body></html>",
        )
        .unwrap();

        let recv = UdpSocket::bind("127.0.0.1:0").unwrap();
        let sender = UdpSender::connect(&recv.local_addr().unwrap().to_string()).unwrap();
        let (tlm_tx, _) = broadcast::channel(4);
        let app = build_app(sender, tlm_tx, Some(dir.to_str().unwrap()));

        let res = app
            .oneshot(
                Request::builder()
                    .uri("/telemetry")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(res.status(), StatusCode::OK);
        let body = res.into_body().collect().await.unwrap().to_bytes();
        let s = String::from_utf8_lossy(&body);
        assert!(
            s.contains("spa"),
            "expected index.html body for /telemetry, got {s:?}"
        );
    }

    #[tokio::test]
    async fn api_error_into_response_bad_request_and_io() {
        let bad = ApiError::BadRequest("bad".into()).into_response();
        assert_eq!(bad.status(), StatusCode::BAD_REQUEST);
        let io = ApiError::Io("disk".into()).into_response();
        assert_eq!(io.status(), StatusCode::INTERNAL_SERVER_ERROR);
        let up = ApiError::UpstreamUnavailable("no ci_lab".into()).into_response();
        assert_eq!(up.status(), StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn api_send_rejects_bad_json() {
        let recv = UdpSocket::bind("127.0.0.1:0").expect("bind");
        let addr = recv.local_addr().expect("addr");
        let sender = UdpSender::connect(&addr.to_string()).expect("connect");
        let (tlm_tx, _) = broadcast::channel(4);
        let app = build_app(sender, tlm_tx, None);

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
