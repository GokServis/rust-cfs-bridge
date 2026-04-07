//! Long-lived HTTP server: JSON → CCSDS → UDP to CI_LAB.
//!
//! CLI mode:
//!   `cargo run --bin bridge_server -- brain-upload`

use rust_bridge::brain_upload::{run_master_brain_upload, BrainUploadConfig};
use rust_bridge::server::run;
use rust_bridge::tlm::udp_task::run_udp_telemetry_listener_supervised;
use rust_bridge::{TlmEvent, UdpSender};
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() >= 2 && args[1] == "brain-upload" {
        // Minimal "driver" mode: bring up telemetry listener + command verifier, run one upload, exit.
        let udp_target =
            std::env::var("BRIDGE_UDP_TARGET").unwrap_or_else(|_| "127.0.0.1:1234".to_string());
        let tlm_bind = std::env::var("BRIDGE_TLM_BIND")
            .unwrap_or_else(|_| rust_bridge::BRIDGE_TLM_DEFAULT_BIND.to_string());
        let sender = UdpSender::connect(&udp_target)?;
        let (tlm_tx, _tlm_rx) = broadcast::channel::<TlmEvent>(256);
        let tlm_addr: std::net::SocketAddr = tlm_bind.parse()?;

        let state = Arc::new(rust_bridge::server::AppState {
            udp: Mutex::new(sender),
            tlm_tx: tlm_tx.clone(),
            pending_cmd: tokio::sync::Mutex::new(None),
            last_es_counters: tokio::sync::Mutex::new((0, 0)),
        });

        tokio::spawn(run_udp_telemetry_listener_supervised(
            tlm_addr,
            tlm_tx.clone(),
            None,
        ));
        tokio::spawn(rust_bridge::server::run_command_verifier(state.clone()));

        let cfg = BrainUploadConfig::default();
        run_master_brain_upload(cfg, state)
            .await
            .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { e.into() })?;
        Ok(())
    } else {
        run().await
    }
}
