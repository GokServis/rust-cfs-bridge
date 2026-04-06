//! JSON ↔ CCSDS primary-header bridge with a bridge-specific CRC-16 trailer.
//!
//! On-wire layout: `[ 6-byte primary header ][ N-byte payload ][ CRC-16 BE ]`.
//! The CRC is **CRC-16/CCITT-FALSE** (poly `0x1021`, init `0xFFFF`) over the first `6 + N` bytes.
//! Confirm parameters against your mission ICD (`TODO(ICD)`).

mod udp;

#[cfg(feature = "server")]
pub mod server;

pub use udp::UdpSender;

use serde::{Deserialize, Serialize};
use thiserror::Error;

// --- Mission ID mapping (cFS Software Bus vs CCSDS wire) ---
/// cFS Software Bus MsgId for bridge packets (subscription / CI_LAB forward). **Not** placed in the CCSDS APID field.
/// Must not use `0x1806` — that is `CFE_ES_CMD_MID` on default missions; use a free CMD topic (e.g. `0x18F0`).
pub const BRIDGE_SB_MSGID_VALUE: u16 = 0x18F0;
/// On-wire CCSDS primary-header APID (11 bits) for bridge dictionary commands.
pub const BRIDGE_WIRE_APID: u16 = 0x006;

/// Human-readable command name → payload rules and fixed wire APID.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BridgeCommandSpec {
    /// Always [`BRIDGE_WIRE_APID`] for current dictionary entries.
    pub wire_apid: u16,
    pub default_payload: &'static [u8],
    pub payload_len: PayloadLenRule,
}

/// Allowed user payload sizes for a named command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PayloadLenRule {
    Exact(usize),
    Range { min: usize, max: usize },
}

impl BridgeCommandSpec {
    /// Built-in dictionary: heartbeat telecommand (3-byte payload, 11-byte wire with CRC).
    pub const CMD_HEARTBEAT: Self = Self {
        wire_apid: BRIDGE_WIRE_APID,
        default_payload: &[0xC0, 0xFF, 0xEE],
        payload_len: PayloadLenRule::Exact(3),
    };

    fn validate_payload_len(&self, command_name: &str, len: usize) -> Result<(), BridgeError> {
        match self.payload_len {
            PayloadLenRule::Exact(n) if len == n => Ok(()),
            PayloadLenRule::Exact(n) => Err(BridgeError::PayloadConstraint {
                command: command_name.to_string(),
                expected: n,
                got: len,
            }),
            PayloadLenRule::Range { min, max } if (min..=max).contains(&len) => Ok(()),
            PayloadLenRule::Range { min, max } => {
                Err(BridgeError::PayloadConstraintRange { min, max, got: len })
            }
        }
    }
}

/// User-facing metadata for a named dictionary command (mirrors [`BridgeCommandSpec`]).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CommandMetadata {
    pub name: &'static str,
    pub title: &'static str,
    pub description: &'static str,
    pub wire_apid: u16,
    pub software_bus_msg_id: u16,
    pub payload: PayloadConstraintJson,
}

/// JSON-friendly payload size rule for forms and client-side checks.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PayloadConstraintJson {
    Exact { bytes: usize },
    Range { min: usize, max: usize },
}

impl PayloadConstraintJson {
    fn from_rule(rule: PayloadLenRule) -> Self {
        match rule {
            PayloadLenRule::Exact(n) => PayloadConstraintJson::Exact { bytes: n },
            PayloadLenRule::Range { min, max } => PayloadConstraintJson::Range { min, max },
        }
    }
}

/// Entries for the HTTP `GET /api/commands` response and for the UI dictionary.
pub fn command_dictionary_entries() -> Vec<CommandMetadata> {
    vec![CommandMetadata {
        name: "CMD_HEARTBEAT",
        title: "Heartbeat",
        description: "Sample telecommand to validate the bridge. The UDP datagram uses CCSDS APID 0x006 on the wire; CI_LAB then publishes a Software Bus message with MsgId 0x18F0 (different roles, not the same field).",
        wire_apid: BRIDGE_WIRE_APID,
        software_bus_msg_id: BRIDGE_SB_MSGID_VALUE,
        payload: PayloadConstraintJson::from_rule(BridgeCommandSpec::CMD_HEARTBEAT.payload_len),
    }]
}

/// Resolves a command name to a [`SpaceCommand`] using the static dictionary.
pub fn command_dictionary_resolve(
    name: &str,
    sequence_count: u16,
    payload_override: Option<Vec<u8>>,
) -> Result<SpaceCommand, BridgeError> {
    let spec = match name {
        "CMD_HEARTBEAT" => BridgeCommandSpec::CMD_HEARTBEAT,
        _ => return Err(BridgeError::UnknownCommand(name.to_string())),
    };

    if sequence_count > 0x3FFF {
        return Err(BridgeError::SequenceCountOutOfRange(sequence_count));
    }

    let payload = match payload_override {
        Some(p) => {
            spec.validate_payload_len(name, p.len())?;
            p
        }
        None => {
            spec.validate_payload_len(name, spec.default_payload.len())?;
            spec.default_payload.to_vec()
        }
    };

    Ok(SpaceCommand {
        apid: spec.wire_apid,
        sequence_count,
        payload,
    })
}

/// CRC-16/CCITT-FALSE over `data`.
///
/// Parameters: width 16, poly `0x1021`, init `0xFFFF`, no refin/refout, xorout `0`.
/// **TODO(ICD):** Replace or verify against mission checksum specification.
pub fn compute_crc16_ccitt(data: &[u8]) -> u16 {
    let mut crc = 0xFFFFu16;
    for &b in data {
        crc ^= (b as u16) << 8;
        for _ in 0..8 {
            if (crc & 0x8000) != 0 {
                crc = (crc << 1) ^ 0x1021;
            } else {
                crc <<= 1;
            }
        }
    }
    crc
}

/// Command input deserialized from JSON (`payload` is hex).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct SpaceCommand {
    pub apid: u16,
    pub sequence_count: u16,
    #[serde(deserialize_with = "deserialize_hex_payload")]
    pub payload: Vec<u8>,
}

/// JSON shape: either a dictionary command name or legacy `apid` + hex `payload`.
#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum CommandJson {
    Named {
        command: String,
        #[serde(default)]
        sequence_count: u16,
        #[serde(default)]
        payload: Option<String>,
    },
    Legacy {
        apid: u16,
        sequence_count: u16,
        #[serde(deserialize_with = "deserialize_hex_payload")]
        payload: Vec<u8>,
    },
}

fn deserialize_hex_payload<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    decode_hex(&s).map_err(serde::de::Error::custom)
}

fn decode_hex(s: &str) -> Result<Vec<u8>, String> {
    let s = s.trim();
    if s.is_empty() {
        return Ok(Vec::new());
    }
    if !s.len().is_multiple_of(2) {
        return Err("hex payload must have an even number of digits".into());
    }
    let mut out = Vec::with_capacity(s.len() / 2);
    for chunk in s.as_bytes().chunks(2) {
        let hi = hex_val(chunk[0])?;
        let lo = hex_val(chunk[1])?;
        out.push((hi << 4) | lo);
    }
    Ok(out)
}

fn hex_val(b: u8) -> Result<u8, String> {
    match b {
        b'0'..=b'9' => Ok(b - b'0'),
        b'a'..=b'f' => Ok(b - b'a' + 10),
        b'A'..=b'F' => Ok(b - b'A' + 10),
        _ => Err(format!("invalid hex digit: {:?}", b as char)),
    }
}

#[derive(Debug, Error)]
pub enum BridgeError {
    #[error("unknown command: {0}")]
    UnknownCommand(String),
    #[error("payload length invalid for {command}: expected {expected} bytes, got {got}")]
    PayloadConstraint {
        command: String,
        expected: usize,
        got: usize,
    },
    #[error("payload length invalid: expected {min}..={max} bytes, got {got}")]
    PayloadConstraintRange { min: usize, max: usize, got: usize },
    #[error("APID must be <= 0x7FF (got {0})")]
    ApidOutOfRange(u16),
    #[error("sequence count must be <= 0x3FFF (got {0})")]
    SequenceCountOutOfRange(u16),
    #[error("truncated packet buffer")]
    Truncated,
    #[error("packet length mismatch: expected {expected} bytes, got {got}")]
    LengthMismatch { expected: usize, got: usize },
    #[error("checksum mismatch: expected 0x{expected:04x}, got 0x{got:04x}")]
    ChecksumMismatch { expected: u16, got: u16 },
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("hex payload: {0}")]
    HexPayload(String),
}

/// CCSDS space packet primary header fields, payload, and expected CRC (from wire or validation).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CcsdsPacket {
    pub version: u8,
    /// 0 = telemetry, 1 = telecommand (CCSDS type bit).
    pub packet_type: u8,
    pub secondary_header_flag: bool,
    pub apid: u16,
    pub sequence_flags: u8,
    pub sequence_count: u16,
    pub payload: Vec<u8>,
    /// CRC-16/CCITT-FALSE over `primary_header_bytes() || payload`; trailing two bytes on the wire.
    pub crc16_ccitt: u16,
}

impl CcsdsPacket {
    /// Default bridge packet: TC, no secondary header, unsegmented sequence flags.
    pub fn from_command(cmd: &SpaceCommand) -> Result<Self, BridgeError> {
        if cmd.apid > 0x7FF {
            return Err(BridgeError::ApidOutOfRange(cmd.apid));
        }
        if cmd.sequence_count > 0x3FFF {
            return Err(BridgeError::SequenceCountOutOfRange(cmd.sequence_count));
        }
        Ok(Self {
            version: 0,
            packet_type: 1,
            secondary_header_flag: false,
            apid: cmd.apid,
            sequence_flags: 0b11,
            sequence_count: cmd.sequence_count,
            payload: cmd.payload.clone(),
            crc16_ccitt: 0,
        })
    }

    /// Builds the 6-byte primary header (big-endian 16-bit words per CCSDS).
    pub fn primary_header_bytes(&self) -> [u8; 6] {
        let w0 = ((self.version as u16 & 0x7) << 13)
            | ((self.packet_type as u16 & 1) << 12)
            | ((self.secondary_header_flag as u16) << 11)
            | (self.apid & 0x7FF);
        let w1 = ((self.sequence_flags as u16 & 0x3) << 14) | (self.sequence_count & 0x3FFF);
        let packet_data_length = packet_data_length_field(self.payload.len());
        let w2 = packet_data_length;
        let mut out = [0u8; 6];
        out[0..2].copy_from_slice(&w0.to_be_bytes());
        out[2..4].copy_from_slice(&w1.to_be_bytes());
        out[4..6].copy_from_slice(&w2.to_be_bytes());
        out
    }

    /// Serializes `primary_header || payload || CRC-16 BE`. CRC is always computed from header+payload.
    pub fn to_bytes(&self) -> Vec<u8> {
        let header = self.primary_header_bytes();
        let mut buf = Vec::with_capacity(6 + self.payload.len() + 2);
        buf.extend_from_slice(&header);
        buf.extend_from_slice(&self.payload);
        let crc = compute_crc16_ccitt(&buf);
        buf.extend_from_slice(&crc.to_be_bytes());
        buf
    }

    /// Parses wire buffer and verifies checksum.
    pub fn from_bytes(data: &[u8]) -> Result<Self, BridgeError> {
        if data.len() < 8 {
            return Err(BridgeError::Truncated);
        }
        let w0 = u16::from_be_bytes([data[0], data[1]]);
        let w1 = u16::from_be_bytes([data[2], data[3]]);
        let w2 = u16::from_be_bytes([data[4], data[5]]);

        let version = ((w0 >> 13) & 0x7) as u8;
        let packet_type = ((w0 >> 12) & 1) as u8;
        let secondary_header_flag = ((w0 >> 11) & 1) != 0;
        let apid = w0 & 0x7FF;
        let sequence_flags = ((w1 >> 14) & 0x3) as u8;
        let sequence_count = w1 & 0x3FFF;

        let payload_len = payload_len_from_data_length_field(w2);
        let total = 6 + payload_len + 2;
        if data.len() != total {
            return Err(BridgeError::LengthMismatch {
                expected: total,
                got: data.len(),
            });
        }

        let payload = data[6..6 + payload_len].to_vec();
        let crc16_ccitt = u16::from_be_bytes([data[6 + payload_len], data[6 + payload_len + 1]]);

        let pkt = Self {
            version,
            packet_type,
            secondary_header_flag,
            apid,
            sequence_flags,
            sequence_count,
            payload,
            crc16_ccitt,
        };
        pkt.validate_checksum()?;
        Ok(pkt)
    }

    /// Recomputes CRC over `primary_header_bytes() || payload` and compares to `crc16_ccitt`.
    pub fn validate_checksum(&self) -> Result<(), BridgeError> {
        let header = self.primary_header_bytes();
        let mut covered = Vec::with_capacity(6 + self.payload.len());
        covered.extend_from_slice(&header);
        covered.extend_from_slice(&self.payload);
        let expected = compute_crc16_ccitt(&covered);
        if expected == self.crc16_ccitt {
            Ok(())
        } else {
            Err(BridgeError::ChecksumMismatch {
                expected,
                got: self.crc16_ccitt,
            })
        }
    }
}

/// CCSDS Packet Data Length field: `N - 1` for `N > 0`; `0xFFFF` means zero-byte data field.
fn packet_data_length_field(payload_len: usize) -> u16 {
    if payload_len == 0 {
        0xFFFF
    } else {
        (payload_len - 1) as u16
    }
}

fn payload_len_from_data_length_field(field: u16) -> usize {
    if field == 0xFFFF {
        0
    } else {
        field as usize + 1
    }
}

impl SpaceCommand {
    /// Parses JSON: legacy `{ "apid", "sequence_count", "payload" (hex) }` or
    /// `{ "command": "CMD_HEARTBEAT", "sequence_count"?, "payload"? (hex) }`.
    pub fn from_json(s: &str) -> Result<Self, BridgeError> {
        let j: CommandJson = serde_json::from_str(s)?;
        match j {
            CommandJson::Named {
                command,
                sequence_count,
                payload,
            } => {
                let override_bytes = match payload {
                    None => None,
                    Some(hex) => Some(decode_hex(&hex).map_err(BridgeError::HexPayload)?),
                };
                command_dictionary_resolve(&command, sequence_count, override_bytes)
            }
            CommandJson::Legacy {
                apid,
                sequence_count,
                payload,
            } => Ok(Self {
                apid,
                sequence_count,
                payload,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn header_construction_exact_six_bytes() {
        let cmd = SpaceCommand {
            apid: 0x100,
            sequence_count: 0x123,
            payload: vec![1, 2, 3, 4],
        };
        let pkt = CcsdsPacket::from_command(&cmd).unwrap();
        let h = pkt.primary_header_bytes();
        // word0: ver=0, type=1, sec=0, apid=0x100 -> 0x1100
        // word1: flags=3, seq=0x123 -> 0xC123
        // word2: N-1 = 3 -> 0x0003
        assert_eq!(h, [0x11, 0x00, 0xC1, 0x23, 0x00, 0x03]);
    }

    #[test]
    fn packet_data_length_n_minus_one() {
        let cases = [
            (vec![0u8; 0], 0xFFFFu16),
            (vec![0u8; 1], 0u16),
            (vec![0u8; 255], 254u16),
            (vec![0u8; 256], 255u16),
        ];
        for (payload, want_field) in cases {
            let mut pkt = CcsdsPacket::from_command(&SpaceCommand {
                apid: 1,
                sequence_count: 0,
                payload: payload.clone(),
            })
            .unwrap();
            pkt.packet_type = 1;
            let w2 =
                u16::from_be_bytes([pkt.primary_header_bytes()[4], pkt.primary_header_bytes()[5]]);
            assert_eq!(w2, want_field, "payload len {}", payload.len());
        }
    }

    #[test]
    fn packet_assembly_header_payload_crc() {
        let cmd = SpaceCommand {
            apid: 0x42,
            sequence_count: 7,
            payload: vec![0xAA, 0xBB],
        };
        let pkt = CcsdsPacket::from_command(&cmd).unwrap();
        let wire = pkt.to_bytes();
        let header = pkt.primary_header_bytes();
        assert_eq!(&wire[..6], header.as_slice());
        assert_eq!(&wire[6..8], cmd.payload.as_slice());
        let crc = compute_crc16_ccitt(&wire[..8]);
        assert_eq!(wire[8..10], crc.to_be_bytes());
        assert_eq!(wire.len(), 6 + cmd.payload.len() + 2);
    }

    #[test]
    fn validate_checksum_rejects_wrong_crc_field() {
        let cmd = SpaceCommand {
            apid: 1,
            sequence_count: 0,
            payload: vec![0x01],
        };
        let mut pkt = CcsdsPacket::from_command(&cmd).unwrap();
        pkt.crc16_ccitt = 0xBAD0;
        let err = pkt.validate_checksum().unwrap_err();
        assert!(
            matches!(err, BridgeError::ChecksumMismatch { .. }),
            "{err:?}"
        );
    }

    #[test]
    fn test_radiation_bit_flip() {
        let cmd = SpaceCommand {
            apid: 0x7FF,
            sequence_count: 0x3FFF,
            payload: vec![0xDE, 0xAD, 0xBE, 0xEF],
        };
        let pkt = CcsdsPacket::from_command(&cmd).unwrap();
        let mut wire = pkt.to_bytes();
        assert!(CcsdsPacket::from_bytes(&wire).is_ok());
        // Flip one bit in header+payload region (not CRC trailer).
        wire[3] ^= 1;
        let err = CcsdsPacket::from_bytes(&wire).unwrap_err();
        assert!(
            matches!(err, BridgeError::ChecksumMismatch { .. }),
            "expected checksum failure, got {err:?}"
        );
    }

    #[test]
    fn json_malformed_returns_error() {
        let err = SpaceCommand::from_json("not json").unwrap_err();
        assert!(matches!(err, BridgeError::Json(_)));
    }

    #[test]
    fn json_valid_maps_to_space_command() {
        let j = r#"{"apid":100,"sequence_count":5,"payload":"DEADbeef"}"#;
        let cmd = SpaceCommand::from_json(j).unwrap();
        assert_eq!(cmd.apid, 100);
        assert_eq!(cmd.sequence_count, 5);
        assert_eq!(cmd.payload, vec![0xDE, 0xAD, 0xBE, 0xEF]);
    }

    #[test]
    fn json_invalid_hex_errors() {
        let j = r#"{"apid":1,"sequence_count":0,"payload":"GG"}"#;
        let err = SpaceCommand::from_json(j).unwrap_err();
        assert!(matches!(err, BridgeError::Json(_)));
    }

    #[test]
    fn from_bytes_round_trip() {
        let cmd = SpaceCommand {
            apid: 0x55,
            sequence_count: 0x100,
            payload: (0u8..=20).collect(),
        };
        let pkt = CcsdsPacket::from_command(&cmd).unwrap();
        let wire = pkt.to_bytes();
        let back = CcsdsPacket::from_bytes(&wire).unwrap();
        assert_eq!(back.apid, pkt.apid);
        assert_eq!(back.sequence_count, pkt.sequence_count);
        assert_eq!(back.payload, pkt.payload);
        assert_eq!(back.version, pkt.version);
        assert_eq!(back.packet_type, pkt.packet_type);
        assert_eq!(back.secondary_header_flag, pkt.secondary_header_flag);
        assert_eq!(back.sequence_flags, pkt.sequence_flags);
        let crc = compute_crc16_ccitt(&wire[..wire.len() - 2]);
        assert_eq!(back.crc16_ccitt, crc);
    }

    #[test]
    fn from_command_rejects_apid_out_of_range() {
        let cmd = SpaceCommand {
            apid: 0x800,
            sequence_count: 0,
            payload: vec![],
        };
        let err = CcsdsPacket::from_command(&cmd).unwrap_err();
        assert!(matches!(err, BridgeError::ApidOutOfRange(0x800)));
    }

    #[test]
    fn from_command_rejects_sequence_out_of_range() {
        let cmd = SpaceCommand {
            apid: 0,
            sequence_count: 0x4000,
            payload: vec![],
        };
        let err = CcsdsPacket::from_command(&cmd).unwrap_err();
        assert!(matches!(err, BridgeError::SequenceCountOutOfRange(0x4000)));
    }

    #[test]
    fn from_bytes_truncated_when_too_short() {
        assert!(matches!(
            CcsdsPacket::from_bytes(&[]).unwrap_err(),
            BridgeError::Truncated
        ));
        assert!(matches!(
            CcsdsPacket::from_bytes(&[0u8; 7]).unwrap_err(),
            BridgeError::Truncated
        ));
    }

    #[test]
    fn from_bytes_length_mismatch() {
        let cmd = SpaceCommand {
            apid: 1,
            sequence_count: 0,
            payload: vec![0xAB, 0xCD],
        };
        let pkt = CcsdsPacket::from_command(&cmd).unwrap();
        let mut wire = pkt.to_bytes();
        // Drop CRC trailer -> total shorter than header + payload + 2
        wire.pop();
        let err = CcsdsPacket::from_bytes(&wire).unwrap_err();
        match err {
            BridgeError::LengthMismatch { expected, got } => {
                assert_eq!(expected, 10);
                assert_eq!(got, 9);
            }
            e => panic!("expected LengthMismatch, got {e:?}"),
        }
        // Too long
        wire = pkt.to_bytes();
        wire.push(0);
        let err = CcsdsPacket::from_bytes(&wire).unwrap_err();
        match err {
            BridgeError::LengthMismatch { expected, got } => {
                assert_eq!(expected, 10);
                assert_eq!(got, 11);
            }
            e => panic!("expected LengthMismatch, got {e:?}"),
        }
    }

    #[test]
    fn validate_checksum_ok_when_crc_matches() {
        let cmd = SpaceCommand {
            apid: 2,
            sequence_count: 1,
            payload: vec![0x00],
        };
        let pkt = CcsdsPacket::from_command(&cmd).unwrap();
        let wire = pkt.to_bytes();
        let parsed = CcsdsPacket::from_bytes(&wire).unwrap();
        parsed.validate_checksum().expect("checksum should match");
    }

    #[test]
    fn json_empty_hex_payload() {
        let j = r#"{"apid":0,"sequence_count":0,"payload":""}"#;
        let cmd = SpaceCommand::from_json(j).unwrap();
        assert!(cmd.payload.is_empty());
    }

    #[test]
    fn json_odd_length_hex_errors() {
        let j = r#"{"apid":0,"sequence_count":0,"payload":"ABC"}"#;
        let err = SpaceCommand::from_json(j).unwrap_err();
        assert!(matches!(err, BridgeError::Json(_)));
    }

    #[test]
    fn empty_payload_round_trip_from_bytes() {
        let cmd = SpaceCommand {
            apid: 0,
            sequence_count: 0,
            payload: vec![],
        };
        let pkt = CcsdsPacket::from_command(&cmd).unwrap();
        let wire = pkt.to_bytes();
        assert_eq!(wire.len(), 8);
        let back = CcsdsPacket::from_bytes(&wire).unwrap();
        assert!(back.payload.is_empty());
        assert_eq!(back.apid, 0);
    }

    #[test]
    fn dictionary_cmd_heartbeat_maps_to_wire_apid_and_crc() {
        let cmd = command_dictionary_resolve("CMD_HEARTBEAT", 0, None).unwrap();
        assert_eq!(cmd.apid, BRIDGE_WIRE_APID);
        assert_eq!(cmd.payload, vec![0xC0, 0xFF, 0xEE]);
        let pkt = CcsdsPacket::from_command(&cmd).unwrap();
        let wire = pkt.to_bytes();
        assert_eq!(wire.len(), 11);
        let crc = compute_crc16_ccitt(&wire[..9]);
        assert_eq!(wire[9..11], crc.to_be_bytes());
    }

    #[test]
    fn sb_msgid_constant_is_not_ccsds_apid_on_wire() {
        // BRIDGE_SB_MSGID_VALUE is the cFS SB MsgId; the CCSDS header carries only 11-bit APID (here 0x006).
        let cmd = command_dictionary_resolve("CMD_HEARTBEAT", 0, None).unwrap();
        let pkt = CcsdsPacket::from_command(&cmd).unwrap();
        let w0 = u16::from_be_bytes([pkt.primary_header_bytes()[0], pkt.primary_header_bytes()[1]]);
        let apid_on_wire = w0 & 0x7FF;
        assert_eq!(apid_on_wire, 0x006);
        assert_ne!(apid_on_wire as u32, BRIDGE_SB_MSGID_VALUE as u32);
    }

    #[test]
    fn dictionary_unknown_command_errors() {
        let err = command_dictionary_resolve("CMD_UNKNOWN", 0, None).unwrap_err();
        assert!(matches!(err, BridgeError::UnknownCommand(_)));
    }

    #[test]
    fn dictionary_payload_wrong_length_errors() {
        let err = command_dictionary_resolve("CMD_HEARTBEAT", 0, Some(vec![0x01])).unwrap_err();
        assert!(matches!(err, BridgeError::PayloadConstraint { .. }));
    }

    #[test]
    fn json_named_command_parses() {
        let j = r#"{"command":"CMD_HEARTBEAT","sequence_count":0}"#;
        let cmd = SpaceCommand::from_json(j).unwrap();
        assert_eq!(cmd.apid, BRIDGE_WIRE_APID);
        assert_eq!(cmd.payload, vec![0xC0, 0xFF, 0xEE]);
    }

    #[test]
    fn json_named_command_with_hex_payload_override() {
        let j = r#"{"command":"CMD_HEARTBEAT","sequence_count":1,"payload":"010203"}"#;
        let cmd = SpaceCommand::from_json(j).unwrap();
        assert_eq!(cmd.sequence_count, 1);
        assert_eq!(cmd.payload, vec![1, 2, 3]);
    }

    #[test]
    fn command_dictionary_entries_includes_heartbeat() {
        let e = command_dictionary_entries();
        assert!(e.iter().any(|c| c.name == "CMD_HEARTBEAT"));
    }
}
