//! CFDP Class 1 sender ("slicer") for a single file/image.

use crate::cfdp::pdu::{encode_eof_pdu, encode_file_data_pdu, encode_metadata_pdu, Mib, PduError};

/// CF "CRC" (actually a modular checksum) as implemented by CF's `CF_CRC_*` routines.
///
/// This is a streaming sum of 32-bit big-endian words, with the final partial word
/// left-aligned and zero-padded.
pub fn cf_modular_checksum_u32(image: &[u8]) -> u32 {
    let mut result: u32 = 0;
    let mut working: u32 = 0;
    let mut index: u8 = 0;

    for &b in image {
        working = (working << 8) | (b as u32);
        index += 1;
        if index == 4 {
            result = result.wrapping_add(working);
            working = 0;
            index = 0;
        }
    }

    if index != 0 {
        let shift = 8 * (4 - index as u32);
        result = result.wrapping_add(working << shift);
    }

    result
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CfdpFileTransferConfig {
    pub source_eid: u64,
    pub destination_eid: u64,
    pub transaction_seq: u64,
    pub src_filename: String,
    pub dst_filename: String,
    /// Checksum type (low nibble in Metadata); start with 0 (null) for lab.
    pub checksum_type: u8,
    pub closure_requested: bool,
    /// Max bytes of file data per File Data PDU (excluding PDU header + offset).
    pub max_segment_data: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SendStep {
    Metadata,
    FileData { offset: usize, len: usize },
    Eof,
}

/// Produces an ordered set of PDUs to transmit a single image using Class 1.
pub fn build_class1_pdus(
    mib: &Mib,
    cfg: &CfdpFileTransferConfig,
    image: &[u8],
    eof_checksum: u32,
) -> Result<Vec<(SendStep, Vec<u8>)>, PduError> {
    let mut out: Vec<(SendStep, Vec<u8>)> = Vec::new();

    // Metadata
    let md = encode_metadata_pdu(
        mib,
        cfg.source_eid,
        cfg.destination_eid,
        cfg.transaction_seq,
        image.len() as u64,
        cfg.checksum_type,
        cfg.closure_requested,
        &cfg.src_filename,
        &cfg.dst_filename,
    )?;
    out.push((SendStep::Metadata, md));

    // File data
    let mut off = 0usize;
    while off < image.len() {
        let take = (image.len() - off).min(cfg.max_segment_data);
        let seg = encode_file_data_pdu(
            mib,
            cfg.source_eid,
            cfg.destination_eid,
            cfg.transaction_seq,
            off as u64,
            &image[off..off + take],
        )?;
        out.push((
            SendStep::FileData {
                offset: off,
                len: take,
            },
            seg,
        ));
        off += take;
    }

    // EOF (condition code 0 = no error)
    let eof = encode_eof_pdu(
        mib,
        cfg.source_eid,
        cfg.destination_eid,
        cfg.transaction_seq,
        0,
        eof_checksum,
        image.len() as u64,
    )?;
    out.push((SendStep::Eof, eof));

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cfdp::pdu::Mib;

    #[test]
    fn modular_checksum_matches_cf_algorithm_examples() {
        assert_eq!(cf_modular_checksum_u32(&[]), 0);
        assert_eq!(cf_modular_checksum_u32(&[0x01]), 0x0100_0000);
        assert_eq!(cf_modular_checksum_u32(&[0x01, 0x02]), 0x0102_0000);
        assert_eq!(cf_modular_checksum_u32(&[0x01, 0x02, 0x03]), 0x0102_0300);
        assert_eq!(
            cf_modular_checksum_u32(&[0x01, 0x02, 0x03, 0x04]),
            0x0102_0304
        );
        assert_eq!(
            cf_modular_checksum_u32(&[0x01, 0x02, 0x03, 0x04, 0x05]),
            0x0102_0304u32.wrapping_add(0x0500_0000)
        );
    }

    #[test]
    fn slicer_covers_entire_buffer_with_monotonic_offsets() {
        let mib = Mib::default();
        let cfg = CfdpFileTransferConfig {
            source_eid: 25,
            destination_eid: 23,
            transaction_seq: 0x01020304,
            src_filename: "weights.tbl".into(),
            dst_filename: "/cf/ai_app_weights.tbl".into(),
            checksum_type: 0,
            closure_requested: false,
            max_segment_data: 7,
        };
        let image: Vec<u8> = (0u8..=31).collect();
        let pdus = build_class1_pdus(&mib, &cfg, &image, 0).unwrap();
        assert!(matches!(pdus[0].0, SendStep::Metadata));
        assert!(matches!(pdus.last().unwrap().0, SendStep::Eof));

        let mut saw = vec![false; image.len()];
        let mut last_off = 0usize;
        for (step, pdu) in pdus.iter().skip(1).take(pdus.len() - 2) {
            let SendStep::FileData { offset, len } = step else {
                panic!("expected file data step");
            };
            assert_eq!(*offset, last_off);
            assert!(*len > 0);
            for s in saw.iter_mut().skip(*offset).take(*len) {
                *s = true;
            }
            last_off += *len;
            assert!(!pdu.is_empty());
        }
        assert!(saw.iter().all(|x| *x));
    }
}
