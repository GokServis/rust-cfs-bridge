//! CFDP PDU encoding for compatibility with NASA CF (cFS CFDP app).
//!
//! This module encodes **raw CFDP PDUs** (no CCSDS primary header) suitable for sending
//! over a transport which then encapsulates into CF's SB message (e.g. `CF_PduCmdMsg_t`).
//!
//! Encoding is aligned with the CF codec implementation in `apps/cf/fsw/src/cf_codec.c`:
//! - Header fields order: source_eid, transaction_seq, destination_eid
//! - Length field is the **PDU data field length** (bytes after the common header)
//! - Bitfields in the flags and eid/tsn lengths bytes match CF's `FSV()` usage.

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PduType {
    FileDirective = 0,
    FileData = 1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    TowardReceiver = 0,
    TowardSender = 1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransmissionMode {
    /// Class 1
    Unacknowledged = 0,
    /// Class 2
    Acknowledged = 1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DirectiveCode {
    Eof = 0x04,
    Metadata = 0x07,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mib {
    pub entity_id_len: usize,      // 1..=8
    pub txn_seq_len: usize,        // 1..=8
    pub segmentation_control: u8,  // 0/1 (1 bit)
    pub segment_metadata_flag: u8, // 0/1 (1 bit)
    pub crc_flag: u8,              // 0/1 (1 bit) - PDU CRC present or not
    pub large_file_flag: u8,       // 0/1 (1 bit)
}

impl Default for Mib {
    fn default() -> Self {
        Self {
            entity_id_len: 2,
            txn_seq_len: 4,
            segmentation_control: 0,
            segment_metadata_flag: 0,
            crc_flag: 0,
            large_file_flag: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PduError {
    BadLen(&'static str),
    Truncated,
}

impl fmt::Display for PduError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PduError::BadLen(s) => write!(f, "bad length: {s}"),
            PduError::Truncated => write!(f, "truncated"),
        }
    }
}

impl std::error::Error for PduError {}

fn enc_uint_be(v: u64, len: usize, out: &mut Vec<u8>) -> Result<(), PduError> {
    if !(1..=8).contains(&len) {
        return Err(PduError::BadLen("len must be 1..=8"));
    }
    for i in (0..len).rev() {
        out.push(((v >> (i * 8)) & 0xFF) as u8);
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommonHeader {
    pub version: u8, // CF uses 1
    pub pdu_type: PduType,
    pub direction: Direction,
    pub tx_mode: TransmissionMode,
    pub crc_flag: u8,
    pub large_file_flag: u8,

    pub segmentation_control: u8,
    pub segment_metadata_flag: u8,
    pub entity_id_len: usize,
    pub txn_seq_len: usize,

    pub source_eid: u64,
    pub transaction_seq: u64,
    pub destination_eid: u64,
}

impl CommonHeader {
    pub fn encode_without_length(&self) -> Result<Vec<u8>, PduError> {
        if !(1..=8).contains(&self.entity_id_len) {
            return Err(PduError::BadLen("entity_id_len"));
        }
        if !(1..=8).contains(&self.txn_seq_len) {
            return Err(PduError::BadLen("txn_seq_len"));
        }
        let mut out = Vec::new();

        // flags byte: version(3)<<5 | dir<<3 | type<<4 | mode<<2 | crc<<1 | large<<0
        let mut flags = 0u8;
        flags |= (self.version & 0x07) << 5;
        flags |= (self.pdu_type as u8 & 0x01) << 4;
        flags |= (self.direction as u8 & 0x01) << 3;
        flags |= (self.tx_mode as u8 & 0x01) << 2;
        flags |= (self.crc_flag & 0x01) << 1;
        flags |= self.large_file_flag & 0x01;
        out.push(flags);

        // length: u16 BE, placeholder, filled later (PDU data field length)
        out.extend_from_slice(&0u16.to_be_bytes());

        // eid_tsn_lengths: seg_ctrl<<7 | eid_len-1 (3 bits)<<4 | seg_meta<<3 | tsn_len-1 (3 bits)
        let mut lens = 0u8;
        lens |= (self.segmentation_control & 0x01) << 7;
        lens |= ((self.entity_id_len as u8 - 1) & 0x07) << 4;
        lens |= (self.segment_metadata_flag & 0x01) << 3;
        lens |= (self.txn_seq_len as u8 - 1) & 0x07;
        out.push(lens);

        // variable fields: source_eid, sequence_num, destination_eid
        enc_uint_be(self.source_eid, self.entity_id_len, &mut out)?;
        enc_uint_be(self.transaction_seq, self.txn_seq_len, &mut out)?;
        enc_uint_be(self.destination_eid, self.entity_id_len, &mut out)?;

        Ok(out)
    }
}

fn fill_length_field(pdu: &mut [u8], header_len: usize) -> Result<(), PduError> {
    if pdu.len() < 4 || header_len < 4 || pdu.len() < header_len {
        return Err(PduError::Truncated);
    }
    // CF interprets this as length of content after this header (data field length)
    let data_len = (pdu.len() - header_len) as u16;
    pdu[1..3].copy_from_slice(&data_len.to_be_bytes());
    Ok(())
}

#[allow(clippy::too_many_arguments)] // CFDP metadata PDU packs many wire fields
pub fn encode_metadata_pdu(
    mib: &Mib,
    source_eid: u64,
    destination_eid: u64,
    transaction_seq: u64,
    file_size: u64,
    checksum_type: u8,
    closure_requested: bool,
    src_filename: &str,
    dst_filename: &str,
) -> Result<Vec<u8>, PduError> {
    let hdr = CommonHeader {
        version: 1,
        pdu_type: PduType::FileDirective,
        direction: Direction::TowardReceiver,
        tx_mode: TransmissionMode::Unacknowledged,
        crc_flag: mib.crc_flag,
        large_file_flag: mib.large_file_flag,
        segmentation_control: mib.segmentation_control,
        segment_metadata_flag: mib.segment_metadata_flag,
        entity_id_len: mib.entity_id_len,
        txn_seq_len: mib.txn_seq_len,
        source_eid,
        transaction_seq,
        destination_eid,
    };
    let mut out = hdr.encode_without_length()?;
    let header_len = out.len();

    // File directive header: directive code
    out.push(DirectiveCode::Metadata as u8);

    // Metadata params: closure_requested (bit7) + checksum_type (low nibble)
    let mut b = checksum_type & 0x0F;
    if closure_requested {
        b |= 0x80;
    }
    out.push(b);

    // file size: 4 bytes if small file, else 8 bytes
    if mib.large_file_flag != 0 {
        out.extend_from_slice(&file_size.to_be_bytes());
    } else {
        out.extend_from_slice(&(file_size as u32).to_be_bytes());
    }

    // LV source filename
    let s = src_filename.as_bytes();
    out.push(s.len() as u8);
    out.extend_from_slice(s);
    // LV dest filename
    let d = dst_filename.as_bytes();
    out.push(d.len() as u8);
    out.extend_from_slice(d);

    fill_length_field(&mut out, header_len)?;
    Ok(out)
}

pub fn encode_file_data_pdu(
    mib: &Mib,
    source_eid: u64,
    destination_eid: u64,
    transaction_seq: u64,
    segment_offset: u64,
    data: &[u8],
) -> Result<Vec<u8>, PduError> {
    let hdr = CommonHeader {
        version: 1,
        pdu_type: PduType::FileData,
        direction: Direction::TowardReceiver,
        tx_mode: TransmissionMode::Unacknowledged,
        crc_flag: mib.crc_flag,
        large_file_flag: mib.large_file_flag,
        segmentation_control: mib.segmentation_control,
        segment_metadata_flag: mib.segment_metadata_flag,
        entity_id_len: mib.entity_id_len,
        txn_seq_len: mib.txn_seq_len,
        source_eid,
        transaction_seq,
        destination_eid,
    };
    let mut out = hdr.encode_without_length()?;
    let header_len = out.len();

    // Segment offset: 4 bytes if small file, else 8 bytes
    if mib.large_file_flag != 0 {
        out.extend_from_slice(&segment_offset.to_be_bytes());
    } else {
        out.extend_from_slice(&(segment_offset as u32).to_be_bytes());
    }

    // Segment metadata is omitted (segment_metadata_flag=0 in our lab defaults)
    out.extend_from_slice(data);

    fill_length_field(&mut out, header_len)?;
    Ok(out)
}

pub fn encode_eof_pdu(
    mib: &Mib,
    source_eid: u64,
    destination_eid: u64,
    transaction_seq: u64,
    condition_code: u8,
    file_checksum: u32,
    file_size: u64,
) -> Result<Vec<u8>, PduError> {
    let hdr = CommonHeader {
        version: 1,
        pdu_type: PduType::FileDirective,
        direction: Direction::TowardReceiver,
        tx_mode: TransmissionMode::Unacknowledged,
        crc_flag: mib.crc_flag,
        large_file_flag: mib.large_file_flag,
        segmentation_control: mib.segmentation_control,
        segment_metadata_flag: mib.segment_metadata_flag,
        entity_id_len: mib.entity_id_len,
        txn_seq_len: mib.txn_seq_len,
        source_eid,
        transaction_seq,
        destination_eid,
    };
    let mut out = hdr.encode_without_length()?;
    let header_len = out.len();

    out.push(DirectiveCode::Eof as u8);
    // EOF condition code is upper nibble per CF codec (CC field). Keep other bits 0.
    out.push((condition_code & 0x0F) << 4);
    out.extend_from_slice(&file_checksum.to_be_bytes());
    if mib.large_file_flag != 0 {
        out.extend_from_slice(&file_size.to_be_bytes());
    } else {
        out.extend_from_slice(&(file_size as u32).to_be_bytes());
    }

    fill_length_field(&mut out, header_len)?;
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn header_bitfields_match_cf_codec_layout() {
        let mib = Mib::default();
        let h = CommonHeader {
            version: 1,
            pdu_type: PduType::FileDirective,
            direction: Direction::TowardReceiver,
            tx_mode: TransmissionMode::Unacknowledged,
            crc_flag: 0,
            large_file_flag: 0,
            segmentation_control: 0,
            segment_metadata_flag: 0,
            entity_id_len: 2,
            txn_seq_len: 4,
            source_eid: 0x0019,
            transaction_seq: 0xAABBCCDD,
            destination_eid: 0x0017,
        };
        let enc = h.encode_without_length().unwrap();
        // flags: version=1 => 0b001<<5 = 0x20; type=0; dir=0; mode=0; crc=0; large=0 => 0x20
        assert_eq!(enc[0], 0x20);
        // length placeholder 0
        assert_eq!(&enc[1..3], &[0, 0]);
        // lens: seg_ctrl=0; entity_len-1=1 => 0b001<<4 = 0x10; seg_meta=0; tsn_len-1=3 => 0x03 => 0x13
        assert_eq!(enc[3], 0x13);
        // source_eid (2), tsn (4), dest_eid (2) big-endian
        assert_eq!(&enc[4..6], &0x0019u16.to_be_bytes());
        assert_eq!(&enc[6..10], &0xAABBCCDDu32.to_be_bytes());
        assert_eq!(&enc[10..12], &0x0017u16.to_be_bytes());

        // quick smoke: metadata encodes and sets length field to (total - header_len)
        let p =
            encode_metadata_pdu(&mib, 0x0019, 0x0017, 0xAABBCCDD, 123, 0, false, "a", "b").unwrap();
        let header_len = 4 + mib.entity_id_len + mib.txn_seq_len + mib.entity_id_len;
        let want = (p.len() - header_len) as u16;
        assert_eq!(&p[1..3], &want.to_be_bytes());
    }
}
