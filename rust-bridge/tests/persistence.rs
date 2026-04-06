//! Integration tests for the JSONL telemetry persistence layer.

use std::time::Duration;

use rust_bridge::persistence::{journal_file_name, spawn_journal_writer};

/// The file name should match `tlm-YYYY-MM-DD.jsonl` for a given UTC date.
#[test]
fn journal_file_name_format() {
    let name = journal_file_name(2026, 4, 7);
    assert_eq!(name, "tlm-2026-04-07.jsonl");
}

/// Lines sent through the channel must appear in the journal file.
#[tokio::test]
async fn journal_writer_appends_lines() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().to_owned();

    let tx = spawn_journal_writer(path.clone());

    tx.send("line-one".to_string()).await.unwrap();
    tx.send("line-two".to_string()).await.unwrap();

    // Drop sender so the writer task can flush and exit cleanly.
    drop(tx);

    // Give the writer time to flush.
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Find the journal file.
    let mut found = None;
    for entry in std::fs::read_dir(&path).unwrap() {
        let entry = entry.unwrap();
        if entry.file_name().to_string_lossy().starts_with("tlm-") {
            found = Some(entry.path());
        }
    }
    let file_path = found.expect("journal file not found");
    let content = std::fs::read_to_string(file_path).unwrap();

    assert!(
        content.contains("line-one"),
        "missing line-one in:\n{content}"
    );
    assert!(
        content.contains("line-two"),
        "missing line-two in:\n{content}"
    );
}
