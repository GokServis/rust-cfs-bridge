//! Integration test: command verifier emits CommandAck when ES HK counter increments.

use std::sync::Arc;
use std::time::Duration;

use rust_bridge::server::{AppState, PendingCommand, run_command_verifier};
use rust_bridge::tlm::{TlmEvent, CommandAckResult};
use rust_bridge::tlm::es_hk::{CFE_TLM_HEADER_PREFIX_BYTES, ES_HK_PAYLOAD_BYTES};
use rust_bridge::tlm::classify_datagram;
use rust_bridge::UdpSender;
use tokio::sync::broadcast;

fn make_es_hk_event(cmd_counter: u8, err_counter: u8) -> TlmEvent {
    let total = CFE_TLM_HEADER_PREFIX_BYTES + ES_HK_PAYLOAD_BYTES;
    let mut d = vec![0u8; total];
    let user_len = total - 6;
    d[0..2].copy_from_slice(&0x0800u16.to_be_bytes());
    d[2..4].copy_from_slice(&0xc000u16.to_be_bytes());
    d[4..6].copy_from_slice(&((user_len - 1) as u16).to_be_bytes());
    // payload starts at offset CFE_TLM_HEADER_PREFIX_BYTES
    d[CFE_TLM_HEADER_PREFIX_BYTES] = cmd_counter;
    d[CFE_TLM_HEADER_PREFIX_BYTES + 1] = err_counter;
    classify_datagram(&d, "2026-01-01T00:00:00.000Z".into())
}

#[tokio::test]
async fn command_verifier_emits_accepted_when_cmd_counter_increments() {
    let sink = std::net::UdpSocket::bind("127.0.0.1:0").unwrap();
    let sender = UdpSender::connect(&sink.local_addr().unwrap().to_string()).unwrap();

    let (tlm_tx, _) = broadcast::channel::<TlmEvent>(256);
    let state = Arc::new(AppState {
        udp: tokio::sync::Mutex::new(sender),
        tlm_tx: tlm_tx.clone(),
        pending_cmd: tokio::sync::Mutex::new(Some(PendingCommand {
            name: "CMD_HEARTBEAT".into(),
            sequence_count: 1,
            sent_at: std::time::Instant::now(),
        })),
        last_es_counters: tokio::sync::Mutex::new((0, 0)),
    });

    tokio::spawn(run_command_verifier(state.clone()));
    let mut rx = tlm_tx.subscribe();
    tokio::time::sleep(Duration::from_millis(10)).await;

    // Simulate ES HK: command_counter went from 0 → 1 (accepted)
    tlm_tx.send(make_es_hk_event(1, 0)).unwrap();

    let ack = tokio::time::timeout(Duration::from_secs(2), async {
        loop {
            match rx.recv().await {
                Ok(TlmEvent::CommandAck { result, name, .. }) => {
                    return (result, name);
                }
                Ok(_) => continue,
                Err(_) => panic!("channel closed"),
            }
        }
    })
    .await
    .expect("timed out waiting for CommandAck");

    assert!(matches!(ack.0, CommandAckResult::Accepted), "expected Accepted, got {:?}", ack.0);
    assert_eq!(ack.1, "CMD_HEARTBEAT");
}

#[tokio::test]
async fn command_verifier_emits_rejected_when_err_counter_increments() {
    let sink = std::net::UdpSocket::bind("127.0.0.1:0").unwrap();
    let sender = UdpSender::connect(&sink.local_addr().unwrap().to_string()).unwrap();

    let (tlm_tx, _) = broadcast::channel::<TlmEvent>(256);
    let state = Arc::new(AppState {
        udp: tokio::sync::Mutex::new(sender),
        tlm_tx: tlm_tx.clone(),
        pending_cmd: tokio::sync::Mutex::new(Some(PendingCommand {
            name: "CMD_PING".into(),
            sequence_count: 2,
            sent_at: std::time::Instant::now(),
        })),
        last_es_counters: tokio::sync::Mutex::new((0, 0)),
    });

    tokio::spawn(run_command_verifier(state.clone()));
    let mut rx = tlm_tx.subscribe();
    tokio::time::sleep(Duration::from_millis(10)).await;

    // Simulate ES HK: command_error_counter went from 0 → 1 (rejected)
    tlm_tx.send(make_es_hk_event(0, 1)).unwrap();

    let ack = tokio::time::timeout(Duration::from_secs(2), async {
        loop {
            match rx.recv().await {
                Ok(TlmEvent::CommandAck { result, name, .. }) => {
                    return (result, name);
                }
                Ok(_) => continue,
                Err(_) => panic!("channel closed"),
            }
        }
    })
    .await
    .expect("timed out waiting for CommandAck");

    assert!(matches!(ack.0, CommandAckResult::Rejected), "expected Rejected, got {:?}", ack.0);
    assert_eq!(ack.1, "CMD_PING");
}
