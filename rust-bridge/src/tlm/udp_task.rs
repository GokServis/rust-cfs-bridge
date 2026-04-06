//! Async UDP listener → broadcast [`super::TlmEvent`].

use std::net::SocketAddr;
use std::time::Duration;

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
    run_udp_telemetry_loop(socket, tx).await;
    Ok(())
}

/// Infinite loop: bind with backoff, run receive loop, rebind if the loop exits.
pub async fn run_udp_telemetry_listener_supervised(
    addr: SocketAddr,
    tx: broadcast::Sender<TlmEvent>,
) {
    let mut backoff_ms: u64 = 200;
    loop {
        match UdpSocket::bind(addr).await {
            Ok(socket) => {
                if let Ok(a) = socket.local_addr() {
                    eprintln!("bridge-server: telemetry UDP listening on {a}");
                }
                backoff_ms = 200;
                run_udp_telemetry_loop(socket, tx.clone()).await;
                eprintln!("bridge-server: telemetry UDP receive loop exited; rebinding");
            }
            Err(e) => {
                eprintln!(
                    "bridge-server: telemetry UDP bind {addr} failed ({e}); retrying in {backoff_ms}ms"
                );
                tokio::time::sleep(Duration::from_millis(backoff_ms)).await;
                backoff_ms = (backoff_ms.saturating_mul(2)).min(10_000);
            }
        }
    }
}

/// Forwards datagrams from an already-bound socket (used by tests with `127.0.0.1:0`).
pub(crate) async fn run_udp_telemetry_loop(socket: UdpSocket, tx: broadcast::Sender<TlmEvent>) {
    let mut buf = vec![0u8; 4096];
    loop {
        match socket.recv_from(&mut buf).await {
            Ok((n, _)) => {
                let received_at = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
                let event = classify_datagram(&buf[..n], received_at);
                let _ = tx.send(event);
            }
            Err(e) => {
                eprintln!("bridge-server: telemetry UDP recv_from: {e}");
                tokio::time::sleep(Duration::from_millis(50)).await;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tlm::es_hk::{CFE_TLM_HEADER_PREFIX_BYTES, ES_HK_PAYLOAD_BYTES};
    use crate::tlm::TlmEvent;

    #[tokio::test]
    async fn forwards_datagram_to_broadcast() {
        let socket = UdpSocket::bind("127.0.0.1:0").await.expect("bind");
        let addr = socket.local_addr().expect("addr");
        let (tx, _) = broadcast::channel::<TlmEvent>(8);
        let mut rx = tx.subscribe();
        let h = tokio::spawn(run_udp_telemetry_loop(socket, tx));

        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        let total = CFE_TLM_HEADER_PREFIX_BYTES + ES_HK_PAYLOAD_BYTES;
        let mut pkt = vec![0u8; total];
        let user_len = total - 6;
        let w2 = (user_len - 1) as u16;
        pkt[0..2].copy_from_slice(&0x0800u16.to_be_bytes());
        pkt[2..4].copy_from_slice(&0xc000u16.to_be_bytes());
        pkt[4..6].copy_from_slice(&w2.to_be_bytes());
        pkt[CFE_TLM_HEADER_PREFIX_BYTES] = 0x42;

        let client = UdpSocket::bind("127.0.0.1:0").await.expect("client");
        client.send_to(&pkt, addr).await.expect("send");

        let ev = tokio::time::timeout(std::time::Duration::from_secs(2), rx.recv())
            .await
            .expect("timeout")
            .expect("recv");
        assert!(matches!(ev, TlmEvent::EsHkV1 { .. }));

        h.abort();
        let _ = h.await;
    }
}
