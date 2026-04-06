//! Async UDP listener → broadcast [`super::TlmEvent`].

use std::net::SocketAddr;
use std::time::Duration;

use chrono::Utc;
use socket2::{Domain, Protocol, Socket, Type};
use tokio::net::UdpSocket;
use tokio::sync::broadcast;

use super::{classify_datagram, TlmEvent};

/// Bind a UDP socket and request a 1 MiB receive buffer to absorb telemetry bursts.
/// The OS may cap the actual size (typically at `net.core.rmem_max`); the call is
/// best-effort — bind still succeeds if `set_recv_buffer_size` fails.
fn bind_with_large_recv_buffer(addr: SocketAddr) -> Result<UdpSocket, std::io::Error> {
    let domain = if addr.is_ipv4() {
        Domain::IPV4
    } else {
        Domain::IPV6
    };
    let s2 = Socket::new(domain, Type::DGRAM, Some(Protocol::UDP))?;
    s2.set_recv_buffer_size(1 << 20).ok(); // best-effort; OS may cap
    s2.set_nonblocking(true)?;
    s2.bind(&addr.into())?;
    let std_sock: std::net::UdpSocket = s2.into();
    UdpSocket::from_std(std_sock)
}

/// Binds `addr` and forwards each datagram on `tx` as a [`TlmEvent`].
pub async fn run_udp_telemetry_listener(
    addr: SocketAddr,
    tx: broadcast::Sender<TlmEvent>,
    journal: Option<tokio::sync::mpsc::Sender<String>>,
) -> Result<(), std::io::Error> {
    let socket = bind_with_large_recv_buffer(addr)?;
    run_udp_telemetry_loop(socket, tx, journal).await;
    Ok(())
}

/// Infinite loop: bind with backoff, run receive loop, rebind if the loop exits.
pub async fn run_udp_telemetry_listener_supervised(
    addr: SocketAddr,
    tx: broadcast::Sender<TlmEvent>,
    journal: Option<tokio::sync::mpsc::Sender<String>>,
) {
    let mut backoff_ms: u64 = 200;
    loop {
        match bind_with_large_recv_buffer(addr) {
            Ok(socket) => {
                if let Ok(a) = socket.local_addr() {
                    eprintln!("bridge-server: telemetry UDP listening on {a}");
                }
                backoff_ms = 200;
                run_udp_telemetry_loop(socket, tx.clone(), journal.clone()).await;
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
pub(crate) async fn run_udp_telemetry_loop(
    socket: UdpSocket,
    tx: broadcast::Sender<TlmEvent>,
    journal: Option<tokio::sync::mpsc::Sender<String>>,
) {
    let mut buf = vec![0u8; 4096];
    loop {
        match socket.recv_from(&mut buf).await {
            Ok((n, _)) => {
                let received_at = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
                let event = classify_datagram(&buf[..n], received_at);
                if let Some(ref jw) = journal {
                    if let Ok(json) = serde_json::to_string(&event) {
                        let _ = jw.try_send(json); // non-blocking; drop if channel full
                    }
                }
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

    /// Verifies that `set_recv_buffer_size` succeeds and the resulting OS buffer is
    /// meaningfully large.  Linux doubles the requested value internally, then caps
    /// at `net.core.rmem_max` (default ~208 KB, yielding ~416 KB reported).
    /// We assert ≥ 128 KiB — always achievable on any reasonable kernel — so the
    /// test is robust across environments without requiring elevated `rmem_max`.
    #[test]
    fn udp_recv_buffer_is_at_least_128kb_after_request() {
        let s2 = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP)).unwrap();
        s2.set_recv_buffer_size(1 << 20).unwrap();
        let actual = s2.recv_buffer_size().unwrap();
        assert!(
            actual >= 128 * 1024,
            "recv buffer too small after request: {actual} bytes (expected >= 131072)"
        );
    }

    #[tokio::test]
    async fn forwards_datagram_to_broadcast() {
        let socket = UdpSocket::bind("127.0.0.1:0").await.expect("bind");
        let addr = socket.local_addr().expect("addr");
        let (tx, _) = broadcast::channel::<TlmEvent>(8);
        let mut rx = tx.subscribe();
        let h = tokio::spawn(run_udp_telemetry_loop(socket, tx, None));

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
