//! Async UDP listener → broadcast [`super::TlmEvent`].

use std::net::SocketAddr;

use chrono::Utc;
use tokio::net::UdpSocket;
use tokio::sync::broadcast;

use super::{classify_datagram, TlmEvent};

/// Binds `addr` and forwards each datagram on `tx` as a [`TlmEvent`].
pub async fn run_udp_telemetry_listener(
    addr: SocketAddr,
    tx: broadcast::Sender<TlmEvent>,
) -> Result<(), std::io::Error> {
    let socket = UdpSocket::bind(addr).await?;
    eprintln!("bridge-server: telemetry UDP listening on {addr}");
    let mut buf = vec![0u8; 4096];
    loop {
        let (n, _) = socket.recv_from(&mut buf).await?;
        let received_at = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
        let event = classify_datagram(&buf[..n], received_at);
        let _ = tx.send(event);
    }
}
