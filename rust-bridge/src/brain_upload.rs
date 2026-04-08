//! Master Brain Upload orchestrator.
//!
//! Runs the full sequence:
//! - generate AI_APP weights table image (deterministic golden)
//! - CFDP Class 1 send over UDP 5235 (to udp_cfdp_ingest)
//! - gate on CF retained-file EVS event + CF EOT telemetry
//! - send CFE_TBL LOAD + ACTIVATE over CI_LAB UDP 1234 (bridge wire)

use std::time::{Duration, Instant};

use std::sync::Arc;

use tokio::sync::broadcast;

use crate::ai_app::cfe_tbl_file::build_cfe_table_file;
use crate::ai_app::golden::generate_microgpt_golden;
use crate::ai_app::mission_dims::default_lab_ai_app_dims;
use crate::ai_app::table_image::build_ai_app_weights_table_image;
use crate::ai_app::table_image::AiAppWeights;
use crate::cfdp::pdu::Mib;
use crate::cfdp::sender::{
    build_class1_pdus, cf_modular_checksum_u32, CfdpFileTransferConfig, SendStep,
};
use crate::cfdp::udp::CfdpUdpSender;
use crate::tlm::TlmEvent;
use crate::{command_dictionary_resolve, CcsdsPacket};

#[derive(Debug, Clone)]
pub struct BrainUploadConfig {
    pub cfdp_udp_target: String,   // e.g. 127.0.0.1:5235
    pub ci_lab_udp_target: String, // e.g. 127.0.0.1:1234
    pub dst_filename: String,      // /cf/ai_app_weights.tbl (PSP: /cf -> ./cf; CF tmp uses /cf/tmp)
    pub tbl_name: String,          // AI_APP.WEIGHTS
    pub timeout_file_gate: Duration,
    pub timeout_cmd_ack: Duration,
    pub retries: u32,
    pub pre_cfdp_delay: Duration,
    pub inter_pdu_delay: Duration,
}

impl Default for BrainUploadConfig {
    fn default() -> Self {
        Self {
            cfdp_udp_target: "127.0.0.1:5235".into(),
            ci_lab_udp_target: "127.0.0.1:1234".into(),
            dst_filename: "/cf/ai_app_weights.tbl".into(),
            tbl_name: "AI_APP.WEIGHTS".into(),
            timeout_file_gate: Duration::from_secs(120),
            timeout_cmd_ack: Duration::from_secs(120),
            retries: 3,
            pre_cfdp_delay: Duration::from_secs(2),
            inter_pdu_delay: Duration::from_millis(100),
        }
    }
}

fn now_string() -> String {
    chrono::Utc::now().to_rfc3339()
}

fn progress(tlm_tx: &broadcast::Sender<TlmEvent>, step: &str, detail: impl Into<String>) {
    let _ = tlm_tx.send(TlmEvent::BrainUploadProgress {
        received_at: now_string(),
        step: step.into(),
        detail: detail.into(),
    });
}

async fn backoff_sleep(attempt: u32) {
    // 250ms, 500ms, 1s, 2s, ...
    let ms = 250u64.saturating_mul(1u64 << attempt.min(4));
    tokio::time::sleep(Duration::from_millis(ms)).await;
}

async fn wait_for_es_hk(
    mut rx: broadcast::Receiver<TlmEvent>,
    timeout: Duration,
) -> Result<(), String> {
    let deadline = Instant::now() + timeout;
    loop {
        let now = Instant::now();
        if now >= deadline {
            return Err("timeout waiting for ES HK downlink after enabling TO_LAB".into());
        }
        let remaining = deadline - now;
        let ev = match tokio::time::timeout(remaining, rx.recv()).await {
            Ok(Ok(ev)) => ev,
            Ok(Err(broadcast::error::RecvError::Lagged(_))) => continue,
            Ok(Err(broadcast::error::RecvError::Closed)) => {
                return Err("telemetry channel closed".into())
            }
            Err(_) => continue,
        };
        if matches!(ev, TlmEvent::EsHkV1 { .. }) {
            return Ok(());
        }
    }
}

async fn send_named_cmd_no_ack(
    state: &Arc<crate::server::AppState>,
    name: &str,
) -> Result<(), String> {
    let cmd = command_dictionary_resolve(name, 0, None).map_err(|e| e.to_string())?;
    let packet = CcsdsPacket::from_command(&cmd).map_err(|e| e.to_string())?;
    let guard = state.udp.lock().await;
    guard.send_packet(&packet).map_err(|e| e.to_string())?;
    Ok(())
}

/// CFE_TBL load success text (see `CFE_TBL_SendLoadFileEventHelper` in cFE `cfe_tbl_load.c`).
const EVS_CFE_TBL_LOAD_INTO_WORKING_BUF: &str =
    "Successful load into 'AI_APP.WEIGHTS' working buffer";
/// Alternate success line from `CFE_TBL_LoadContentFromFile` completion path.
const EVS_CFE_TBL_LOADED_FROM_FILE: &str = "Successfully loaded 'AI_APP.WEIGHTS'";
const EVS_AI_APP_VALIDATE_OK: &str = "AI_APP weights table validation success";

async fn wait_for_cf_retained_then_eot_best_effort(
    tlm_tx: &broadcast::Sender<TlmEvent>,
    mut rx: broadcast::Receiver<TlmEvent>,
    dst_filename: &str,
    timeout_retained: Duration,
    timeout_eot_after_retained: Duration,
) -> Result<(), String> {
    let retained_deadline = Instant::now() + timeout_retained;
    let retained_sub = format!("successfully retained file as {}", dst_filename);

    loop {
        let now = Instant::now();
        if now >= retained_deadline {
            return Err(format!(
                "timeout waiting for CF retained-file event for {dst_filename}"
            ));
        }
        let remaining = retained_deadline - now;
        let ev = match tokio::time::timeout(remaining, rx.recv()).await {
            Ok(Ok(ev)) => ev,
            Ok(Err(broadcast::error::RecvError::Lagged(_))) => continue,
            Ok(Err(broadcast::error::RecvError::Closed)) => {
                return Err("telemetry channel closed".into())
            }
            Err(_) => continue,
        };

        if let TlmEvent::EvsLongEventV1 { evs_long_event, .. } = ev {
            if evs_long_event
                .message
                .contains("AI_APP weights table validation failed")
            {
                return Err(format!(
                    "AI_APP validation failure observed: {}",
                    evs_long_event.message
                ));
            }
            if evs_long_event.message.contains(&retained_sub) {
                progress(
                    tlm_tx,
                    "gate",
                    format!("CF retained-file event observed for {dst_filename}"),
                );
                break;
            }
        }
    }

    let eot_deadline = Instant::now() + timeout_eot_after_retained;
    loop {
        let now = Instant::now();
        if now >= eot_deadline {
            progress(
                tlm_tx,
                "gate",
                format!(
                    "CF EOT telemetry not observed within {:?}; proceeding after retained-file gate",
                    timeout_eot_after_retained
                ),
            );
            return Ok(());
        }
        let remaining = eot_deadline - now;
        let ev = match tokio::time::timeout(remaining, rx.recv()).await {
            Ok(Ok(ev)) => ev,
            Ok(Err(broadcast::error::RecvError::Lagged(_))) => continue,
            Ok(Err(broadcast::error::RecvError::Closed)) => {
                return Err("telemetry channel closed".into())
            }
            Err(_) => continue,
        };

        match ev {
            TlmEvent::CfEotV1 { cf_eot, .. } if cf_eot.dst_filename == dst_filename => {
                progress(
                    tlm_tx,
                    "gate",
                    format!("CF EOT telemetry observed for {dst_filename}"),
                );
                return Ok(());
            }
            TlmEvent::EvsLongEventV1 { evs_long_event, .. }
                if evs_long_event
                    .message
                    .contains("AI_APP weights table validation failed") =>
            {
                return Err(format!(
                    "AI_APP validation failure observed: {}",
                    evs_long_event.message
                ));
            }
            _ => {}
        }
    }
}

async fn send_named_cmd_no_pending(
    state: &Arc<crate::server::AppState>,
    name: &str,
) -> Result<(), String> {
    let tlm_tx = &state.tlm_tx;
    progress(tlm_tx, "cmd_send", format!("sending {name}"));
    let cmd = command_dictionary_resolve(name, 0, None).map_err(|e| e.to_string())?;
    let packet = CcsdsPacket::from_command(&cmd).map_err(|e| e.to_string())?;
    let wire_len = 6 + packet.payload.len() + 2;
    let guard = state.udp.lock().await;
    guard.send_packet(&packet).map_err(|e| e.to_string())?;
    drop(guard);
    progress(
        tlm_tx,
        "cmd_send",
        format!("{name} sent ({wire_len} bytes wire)"),
    );
    Ok(())
}

async fn wait_for_evs_substrings(
    mut rx: broadcast::Receiver<TlmEvent>,
    success_any: &[&str],
    timeout: Duration,
    waiting_detail: &str,
    tlm_tx: &broadcast::Sender<TlmEvent>,
) -> Result<(), String> {
    let deadline = Instant::now() + timeout;
    progress(tlm_tx, "tbl_wait", waiting_detail.to_string());
    loop {
        let now = Instant::now();
        if now >= deadline {
            return Err(format!(
                "timeout waiting for EVS ({waiting_detail}); expected one of: {success_any:?}"
            ));
        }
        let remaining = deadline - now;
        let ev = match tokio::time::timeout(remaining, rx.recv()).await {
            Ok(Ok(ev)) => ev,
            Ok(Err(broadcast::error::RecvError::Lagged(_))) => continue,
            Ok(Err(broadcast::error::RecvError::Closed)) => {
                return Err("telemetry channel closed".into())
            }
            Err(_) => continue,
        };
        if let TlmEvent::EvsLongEventV1 { evs_long_event, .. } = ev {
            if evs_long_event
                .message
                .contains("AI_APP weights table validation failed")
            {
                return Err(format!(
                    "AI_APP validation failure observed: {}",
                    evs_long_event.message
                ));
            }
            for needle in success_any {
                if evs_long_event.message.contains(needle) {
                    progress(
                        tlm_tx,
                        "tbl_evs",
                        format!("matched EVS ({waiting_detail}): {needle}"),
                    );
                    if waiting_detail.contains("CFE_TBL load") {
                        eprintln!("BRAIN_UPLOAD_E2E: CFE_TBL_LOAD_EVS_OK");
                    } else if waiting_detail.contains("AI_APP post-activate") {
                        eprintln!("BRAIN_UPLOAD_E2E: AI_APP_VALIDATE_OK");
                    }
                    return Ok(());
                }
            }
        }
    }
}

/// Runs a single master brain upload attempt.
pub async fn run_master_brain_upload(
    cfg: BrainUploadConfig,
    state: Arc<crate::server::AppState>,
) -> Result<(), String> {
    let tlm_tx = state.tlm_tx.clone();
    let tlm_rx = tlm_tx.subscribe();
    progress(&tlm_tx, "start", "master brain upload starting");

    // Ensure downlink is enabled (so our gates/acks can work).
    // NOTE: We cannot rely on ES HK-based acks before TO_LAB is enabled (chicken/egg).
    // So we send the enable command without waiting for `CommandAck`, then wait for ES HK to appear.
    for attempt in 0..=cfg.retries {
        progress(&tlm_tx, "cmd_send", "sending CMD_TO_LAB_ENABLE_OUTPUT");
        if let Err(e) = send_named_cmd_no_ack(&state, "CMD_TO_LAB_ENABLE_OUTPUT").await {
            if attempt < cfg.retries {
                progress(
                    &tlm_tx,
                    "retry",
                    format!("to_lab enable send failed (attempt {attempt}): {e}"),
                );
                backoff_sleep(attempt).await;
                continue;
            }
            return Err(format!("to_lab enable send failed: {e}"));
        }
        let rx = tlm_tx.subscribe();
        match wait_for_es_hk(rx, Duration::from_secs(10)).await {
            Ok(()) => {
                progress(&tlm_tx, "gate", "ES HK downlink observed (TO_LAB enabled)");
                break;
            }
            Err(e) if attempt < cfg.retries => {
                progress(
                    &tlm_tx,
                    "retry",
                    format!("waiting for ES HK failed (attempt {attempt}): {e}"),
                );
                backoff_sleep(attempt).await;
            }
            Err(e) => return Err(e),
        }
    }

    // Build deterministic table image bytes
    progress(&tlm_tx, "table_generate", "generating AI_APP weights image");
    fn layer_slices(v: &[Vec<f64>]) -> Vec<&[f64]> {
        v.iter().map(|x| x.as_slice()).collect()
    }
    let dims = default_lab_ai_app_dims();
    let owned = generate_microgpt_golden(&dims);
    let attn_wq = layer_slices(&owned.attn_wq);
    let attn_wk = layer_slices(&owned.attn_wk);
    let attn_wv = layer_slices(&owned.attn_wv);
    let attn_wo = layer_slices(&owned.attn_wo);
    let mlp_fc1 = layer_slices(&owned.mlp_fc1);
    let mlp_fc2 = layer_slices(&owned.mlp_fc2);
    let w = AiAppWeights {
        wte: &owned.wte,
        wpe: &owned.wpe,
        lm_head: &owned.lm_head,
        attn_wq: &attn_wq,
        attn_wk: &attn_wk,
        attn_wv: &attn_wv,
        attn_wo: &attn_wo,
        mlp_fc1: &mlp_fc1,
        mlp_fc2: &mlp_fc2,
    };
    let raw_image =
        build_ai_app_weights_table_image(&dims, "LAB_GOLDEN", &w).map_err(|e| e.to_string())?;
    let image = build_cfe_table_file(&raw_image, &cfg.tbl_name, "AI_APP weights uplink")
        .map_err(|e| e.to_string())?;

    // CFDP send PDUs
    for attempt in 0..=cfg.retries {
        progress(
            &tlm_tx,
            "cfdp_send",
            format!(
                "sending CFDP PDUs to {} (attempt {}/{})",
                cfg.cfdp_udp_target,
                attempt + 1,
                cfg.retries + 1
            ),
        );
        let res: Result<(), String> = async {
            let sender = CfdpUdpSender::connect(&cfg.cfdp_udp_target).map_err(|e| e.to_string())?;
            let mib = Mib::default();
            let ft = CfdpFileTransferConfig {
                source_eid: 23,
                destination_eid: 25,
                transaction_seq: 0x01020304,
                src_filename: "ai_app_weights.tbl".into(),
                dst_filename: cfg.dst_filename.clone(),
                checksum_type: 0,
                closure_requested: false,
                max_segment_data: 480,
            };
            if cfg.pre_cfdp_delay > Duration::from_millis(0) {
                tokio::time::sleep(cfg.pre_cfdp_delay).await;
            }
            let eof_checksum = cf_modular_checksum_u32(&image);
            progress(
                &tlm_tx,
                "cfdp_send",
                format!("eof_checksum=0x{eof_checksum:08x}"),
            );
            let pdus =
                build_class1_pdus(&mib, &ft, &image, eof_checksum).map_err(|e| e.to_string())?;
            for (step, pdu) in pdus {
                match step {
                    SendStep::Metadata => {}
                    SendStep::FileData { offset, len } => {
                        progress(
                            &tlm_tx,
                            "cfdp_send",
                            format!("filedata offset={offset} len={len}"),
                        );
                    }
                    SendStep::Eof => {}
                }
                sender.send_pdu(&pdu).map_err(|e| e.to_string())?;
                if cfg.inter_pdu_delay > Duration::from_millis(0) {
                    tokio::time::sleep(cfg.inter_pdu_delay).await;
                }
            }
            Ok(())
        }
        .await;

        match res {
            Ok(()) => break,
            Err(e) if attempt < cfg.retries => {
                progress(
                    &tlm_tx,
                    "retry",
                    format!("CFDP send failed (attempt {attempt}): {e}"),
                );
                backoff_sleep(attempt).await;
            }
            Err(e) => return Err(format!("CFDP send failed: {e}")),
        }
    }

    // Gate on CF retained + EOT
    progress(
        &tlm_tx,
        "gate",
        "waiting for CF retained-file + EOT telemetry",
    );
    wait_for_cf_retained_then_eot_best_effort(
        &tlm_tx,
        tlm_rx,
        &cfg.dst_filename,
        cfg.timeout_file_gate,
        Duration::from_secs(10),
    )
    .await?;
    progress(&tlm_tx, "gate", "CF retained-file gate satisfied");

    // Give CF / OSAL a moment to finish flushing the retained file before CFE_TBL opens it (avoids
    // intermittent "not a cFE file type" / wrong ContentType on partial reads).
    tokio::time::sleep(Duration::from_secs(5)).await;

    // CFE_TBL load + activate via CI_LAB — completion is signaled by EVS (not ES HK; see `es_hk` docs).
    progress(&tlm_tx, "tbl", "CFE_TBL load/activate (EVS-gated)");
    for attempt in 0..=cfg.retries {
        let res = async {
            let rx_load = tlm_tx.subscribe();
            send_named_cmd_no_pending(&state, "CMD_CFE_TBL_LOAD_FILE").await?;
            wait_for_evs_substrings(
                rx_load,
                &[
                    EVS_CFE_TBL_LOAD_INTO_WORKING_BUF,
                    EVS_CFE_TBL_LOADED_FROM_FILE,
                ],
                cfg.timeout_cmd_ack,
                "CFE_TBL load into working buffer",
                &tlm_tx,
            )
            .await?;

            let rx_act = tlm_tx.subscribe();
            send_named_cmd_no_pending(&state, "CMD_CFE_TBL_ACTIVATE").await?;
            wait_for_evs_substrings(
                rx_act,
                &[EVS_AI_APP_VALIDATE_OK],
                cfg.timeout_cmd_ack,
                "AI_APP post-activate validation",
                &tlm_tx,
            )
            .await?;
            Ok::<(), String>(())
        }
        .await;

        match res {
            Ok(()) => break,
            Err(e) if attempt < cfg.retries => {
                progress(
                    &tlm_tx,
                    "retry",
                    format!("TBL load/activate failed (attempt {attempt}): {e}"),
                );
                backoff_sleep(attempt).await;
            }
            Err(e) => return Err(format!("TBL load/activate failed: {e}")),
        }
    }

    progress(&tlm_tx, "done", "master brain upload completed");
    eprintln!("BRAIN_UPLOAD_E2E: COMPLETE");
    Ok(())
}
