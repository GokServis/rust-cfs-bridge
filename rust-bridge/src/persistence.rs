//! JSONL telemetry journal: daily-rotating append-only log.
//!
//! Each telemetry event is serialized to a JSON line and sent to the writer
//! task via an mpsc channel, keeping the hot UDP receive path non-blocking.

use std::path::PathBuf;

use chrono::{Datelike, Utc};
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc;

/// Returns the journal file name for the given UTC date components.
///
/// Format: `tlm-YYYY-MM-DD.jsonl`
pub fn journal_file_name(year: i32, month: u32, day: u32) -> String {
    format!("tlm-{year:04}-{month:02}-{day:02}.jsonl")
}

/// Spawns an async task that appends JSON lines to a daily-rotating file.
///
/// Returns the sender end; drop it to stop the writer task after it has flushed
/// all queued lines.
pub fn spawn_journal_writer(dir: PathBuf) -> mpsc::Sender<String> {
    let (tx, mut rx) = mpsc::channel::<String>(1024);
    tokio::spawn(async move {
        while let Some(line) = rx.recv().await {
            let now = Utc::now();
            let fname = journal_file_name(now.year(), now.month(), now.day());
            let file_path = dir.join(&fname);

            match OpenOptions::new()
                .create(true)
                .append(true)
                .open(&file_path)
                .await
            {
                Ok(mut f) => {
                    let mut entry = line;
                    entry.push('\n');
                    if let Err(e) = f.write_all(entry.as_bytes()).await {
                        eprintln!("bridge-server: journal write error ({file_path:?}): {e}");
                    }
                }
                Err(e) => {
                    eprintln!("bridge-server: journal open error ({file_path:?}): {e}");
                }
            }
        }
    });
    tx
}
