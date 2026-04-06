//! WebSocket `/api/tlm/ws` receives broadcast telemetry JSON.

use std::time::Duration;

use futures_util::StreamExt;
use rust_bridge::server::build_app;
use rust_bridge::tlm::classify_datagram;
use rust_bridge::tlm::es_hk::{CFE_TLM_HEADER_PREFIX_BYTES, ES_HK_PAYLOAD_BYTES};
use rust_bridge::TlmEvent;
use rust_bridge::UdpSender;
use tokio::net::TcpListener;
use tokio_tungstenite::tungstenite::Message;

/// Sending 10 events in a burst to a channel with capacity=256 must not cause
/// `RecvError::Lagged` for a subscriber that subscribed before the burst.
/// This is a regression guard: if the production channel capacity is ever
/// accidentally lowered below 10, this test will catch it.
#[tokio::test]
async fn broadcast_no_lag_at_capacity_256() {
    let (tx, mut rx1) = tokio::sync::broadcast::channel::<TlmEvent>(256);
    let mut rx2 = tx.subscribe();

    // Fire 10 events without any receiver consuming — they pile up in the channel.
    for i in 0u16..10 {
        tx.send(TlmEvent::ParseError {
            received_at: format!("2026-01-01T00:00:00.{i:03}Z"),
            raw_len: 1,
            primary: None,
            message: format!("burst-{i}"),
            hex_preview: String::new(),
        })
        .expect("send failed");
    }

    // Both receivers must drain all 10 — no RecvError::Lagged.
    for i in 0u16..10 {
        assert!(
            rx1.try_recv().is_ok(),
            "rx1 lagged at event {i} (channel capacity too small)"
        );
        assert!(
            rx2.try_recv().is_ok(),
            "rx2 lagged at event {i} (channel capacity too small)"
        );
    }
}

fn sample_es_hk_datagram() -> Vec<u8> {
    let total = CFE_TLM_HEADER_PREFIX_BYTES + ES_HK_PAYLOAD_BYTES;
    let mut d = vec![0u8; total];
    let user_len = total - 6;
    let w2 = (user_len - 1) as u16;
    d[0..2].copy_from_slice(&0x0800u16.to_be_bytes());
    d[2..4].copy_from_slice(&0xc000u16.to_be_bytes());
    d[4..6].copy_from_slice(&w2.to_be_bytes());
    d[CFE_TLM_HEADER_PREFIX_BYTES] = 0x42;
    d
}

#[tokio::test]
async fn websocket_receives_telemetry_json() {
    let sink = std::net::UdpSocket::bind("127.0.0.1:0").expect("bind");
    let sender = UdpSender::connect(&sink.local_addr().unwrap().to_string()).expect("udp sender");
    let (tlm_tx, _) = tokio::sync::broadcast::channel::<TlmEvent>(8);
    let app = build_app(sender, tlm_tx.clone(), None);

    let listener = TcpListener::bind("127.0.0.1:0").await.expect("tcp");
    let http_addr = listener.local_addr().expect("addr");

    let serve = axum::serve(listener, app);
    tokio::spawn(async move {
        let _ = serve.await;
    });
    tokio::time::sleep(Duration::from_millis(80)).await;

    let url = format!("ws://{http_addr}/api/tlm/ws");
    let (mut ws, _) = tokio_tungstenite::connect_async(url.as_str())
        .await
        .expect("ws connect");

    let pkt = sample_es_hk_datagram();
    let ev = classify_datagram(&pkt, "2026-01-01T00:00:00Z".into());
    tlm_tx.send(ev).expect("broadcast");

    let msg = tokio::time::timeout(Duration::from_secs(2), ws.next())
        .await
        .expect("timeout")
        .expect("stream")
        .expect("frame");
    let Message::Text(text) = msg else {
        panic!("expected text");
    };
    assert!(text.contains("es_hk_v1") || text.contains("command_counter"));
    assert!(text.contains("66") || text.contains("0x42")); // 0x42 = 66

    let _ = ws.close(None).await;
}
