//! cFE Table Services on-disk layout: `CFE_FS_Header_t` + `CFE_TBL_File_Hdr_t` + raw table image.
//!
//! Matches NASA cFE Draco definitions in `modules/fs/config/default_cfe_fs_filedef.h` and
//! `modules/tbl/config/default_cfe_tbl_extern_typedefs.h` (see `CFE_TBL_ReadHeaders` in
//! `cfe_tbl_load.c`). Table image bytes must begin where CFE_TBL expects the in-memory struct
//! (`AI_APP_WeightsTable_t`), not at file offset 0.
//!
//! **Endianness:** On disk, `CFE_FS_Header_t` and `CFE_TBL_File_Hdr_t` multi-byte fields use the
//! cFE **file** convention (big-endian / network order). `CFE_FS_ReadHeader` byte-swaps into the
//! native CPU layout on little-endian targets (`cfe_fs_api.c`). Compare a mission-built
//! `*.tbl` (e.g. `sch_lab_table.tbl`): the first four bytes are `63 46 45 31`, not LE `31 45 46 63`.

/// Magic in `CFE_FS_Header_t::ContentType` (= `'cFE1'`).
pub const CFE_FS_FILE_CONTENT_ID: u32 = 0x6346_4531;
/// `CFE_FS_SubType_TBL_IMG` — table image file.
pub const CFE_FS_SUBTYPE_TBL_IMG: u32 = 8;

/// Default `CFE_FS_HDR_DESC_MAX_LEN` in mission sample configs.
pub const CFE_FS_HDR_DESC_MAX_LEN: usize = 32;
/// `CFE_MISSION_TBL_MAX_FULL_NAME_LEN` (matches `CMD_CFE_TBL_ACTIVATE_DEFAULT_PAYLOAD` in `lib.rs`).
pub const CFE_MISSION_TBL_MAX_FULL_NAME_LEN: usize = 40;

/// `sizeof(CFE_FS_Header_t)` with 32-byte description field.
pub const CFE_FS_HEADER_BYTES: usize = 8 * 4 + CFE_FS_HDR_DESC_MAX_LEN;
/// `sizeof(CFE_TBL_File_Hdr_t)` with 40-byte `TableName`.
pub const CFE_TBL_FILE_HDR_BYTES: usize = 4 + 4 + 4 + CFE_MISSION_TBL_MAX_FULL_NAME_LEN;

/// Bytes before the raw application table image (`AI_APP_WeightsTable_t` layout).
pub const CFE_TBL_COMBINED_HDR_BYTES: usize = CFE_FS_HEADER_BYTES + CFE_TBL_FILE_HDR_BYTES;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CfeTblFileError {
    TableNameTooLong { max: usize, got: usize },
    DescriptionTooLong { max: usize, got: usize },
    NumBytesOverflow,
}

impl std::fmt::Display for CfeTblFileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CfeTblFileError::TableNameTooLong { max, got } => {
                write!(f, "table name len {got} exceeds max {max}")
            }
            CfeTblFileError::DescriptionTooLong { max, got } => {
                write!(f, "FS description len {got} exceeds max {max}")
            }
            CfeTblFileError::NumBytesOverflow => write!(f, "raw image too large for u32 NumBytes"),
        }
    }
}

impl std::error::Error for CfeTblFileError {}

fn push_u32_be(buf: &mut Vec<u8>, v: u32) {
    buf.extend_from_slice(&v.to_be_bytes());
}

fn push_cfe_fs_description(buf: &mut Vec<u8>, s: &str) -> Result<(), CfeTblFileError> {
    let b = s.as_bytes();
    if b.len() > CFE_FS_HDR_DESC_MAX_LEN {
        return Err(CfeTblFileError::DescriptionTooLong {
            max: CFE_FS_HDR_DESC_MAX_LEN,
            got: b.len(),
        });
    }
    buf.extend_from_slice(b);
    buf.resize(buf.len() + (CFE_FS_HDR_DESC_MAX_LEN - b.len()), 0);
    Ok(())
}

fn push_tbl_table_name(buf: &mut Vec<u8>, s: &str) -> Result<(), CfeTblFileError> {
    let b = s.as_bytes();
    if b.len() > CFE_MISSION_TBL_MAX_FULL_NAME_LEN {
        return Err(CfeTblFileError::TableNameTooLong {
            max: CFE_MISSION_TBL_MAX_FULL_NAME_LEN,
            got: b.len(),
        });
    }
    buf.extend_from_slice(b);
    buf.resize(buf.len() + (CFE_MISSION_TBL_MAX_FULL_NAME_LEN - b.len()), 0);
    Ok(())
}

/// Builds a complete cFE table file: FS header + TBL file header + `raw_table_image`.
///
/// - `qualified_table_name` must match `CFE_TBL_Register` (e.g. `AI_APP.WEIGHTS`).
/// - `raw_table_image` is the in-memory table blob (e.g. from [`super::table_image::build_ai_app_weights_table_image`]).
/// - Spacecraft / processor IDs are zero; mission optional validation lists usually accept any ID when empty.
pub fn build_cfe_table_file(
    raw_table_image: &[u8],
    qualified_table_name: &str,
    fs_description: &str,
) -> Result<Vec<u8>, CfeTblFileError> {
    let num_bytes: u32 = raw_table_image
        .len()
        .try_into()
        .map_err(|_| CfeTblFileError::NumBytesOverflow)?;

    let mut out = Vec::with_capacity(CFE_TBL_COMBINED_HDR_BYTES + raw_table_image.len());

    // CFE_FS_Header_t (on-disk big-endian fields; see module doc)
    push_u32_be(&mut out, CFE_FS_FILE_CONTENT_ID);
    push_u32_be(&mut out, CFE_FS_SUBTYPE_TBL_IMG);
    push_u32_be(&mut out, CFE_FS_HEADER_BYTES as u32);
    push_u32_be(&mut out, 0); // SpacecraftID
    push_u32_be(&mut out, 0); // ProcessorID
    push_u32_be(&mut out, 0); // ApplicationID
    push_u32_be(&mut out, 0); // TimeSeconds
    push_u32_be(&mut out, 0); // TimeSubSeconds
    push_cfe_fs_description(&mut out, fs_description)?;

    // CFE_TBL_File_Hdr_t
    push_u32_be(&mut out, 0); // Reserved
    push_u32_be(&mut out, 0); // Offset — load from start of table image
    push_u32_be(&mut out, num_bytes);
    push_tbl_table_name(&mut out, qualified_table_name)?;

    debug_assert_eq!(out.len(), CFE_TBL_COMBINED_HDR_BYTES);
    out.extend_from_slice(raw_table_image);
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai_app::table_image::{
        build_ai_app_weights_table_image, AiAppDims, AiAppWeights, AI_APP_WEIGHTS_TBL_MAGIC,
    };

    fn dims_default() -> AiAppDims {
        AiAppDims {
            vocab_size: 17,
            n_embd: 16,
            block_size: 16,
            n_head: 4,
            n_layer: 1,
            max_layer: 4,
            mlp_factor: 4,
        }
    }

    fn layer_slices(v: &[Vec<f64>]) -> Vec<&[f64]> {
        v.iter().map(|x| x.as_slice()).collect()
    }

    #[allow(clippy::type_complexity)]
    fn zero_weights(
        d: &AiAppDims,
    ) -> (
        Vec<f64>,
        Vec<f64>,
        Vec<f64>,
        Vec<Vec<f64>>,
        Vec<Vec<f64>>,
        Vec<Vec<f64>>,
        Vec<Vec<f64>>,
        Vec<Vec<f64>>,
        Vec<Vec<f64>>,
    ) {
        let vocab = d.vocab_size as usize;
        let n_embd = d.n_embd as usize;
        let block = d.block_size as usize;
        let max_layer = d.max_layer as usize;
        let mlp_hidden = (d.n_embd * d.mlp_factor) as usize;
        let wte = vec![0.0; vocab * n_embd];
        let wpe = vec![0.0; block * n_embd];
        let lm = vec![0.0; vocab * n_embd];
        let attn = vec![vec![0.0; n_embd * n_embd]; max_layer];
        let mlp1 = vec![vec![0.0; mlp_hidden * n_embd]; max_layer];
        let mlp2 = vec![vec![0.0; n_embd * mlp_hidden]; max_layer];
        (
            wte,
            wpe,
            lm,
            attn.clone(),
            attn.clone(),
            attn.clone(),
            attn,
            mlp1,
            mlp2,
        )
    }

    #[test]
    fn combined_header_size_is_116_bytes() {
        assert_eq!(CFE_FS_HEADER_BYTES, 64);
        assert_eq!(CFE_TBL_FILE_HDR_BYTES, 52);
        assert_eq!(CFE_TBL_COMBINED_HDR_BYTES, 116);
    }

    #[test]
    fn wrapped_file_starts_with_cfe_content_type_and_preserves_table_magic() {
        let d = dims_default();
        let (wte, wpe, lm, wq, wk, wv, wo, fc1, fc2) = zero_weights(&d);
        let w = AiAppWeights {
            wte: &wte,
            wpe: &wpe,
            lm_head: &lm,
            attn_wq: &layer_slices(&wq),
            attn_wk: &layer_slices(&wk),
            attn_wv: &layer_slices(&wv),
            attn_wo: &layer_slices(&wo),
            mlp_fc1: &layer_slices(&fc1),
            mlp_fc2: &layer_slices(&fc2),
        };
        let raw = build_ai_app_weights_table_image(&d, "MISSION_X", &w).unwrap();
        let file = build_cfe_table_file(&raw, "AI_APP.WEIGHTS", "AI_APP weights").unwrap();

        assert_eq!(file[0..4], [0x63, 0x46, 0x45, 0x31]);
        assert_eq!(
            u32::from_be_bytes(file[0..4].try_into().unwrap()),
            CFE_FS_FILE_CONTENT_ID
        );
        assert_eq!(
            u32::from_be_bytes(file[4..8].try_into().unwrap()),
            CFE_FS_SUBTYPE_TBL_IMG
        );
        assert_eq!(
            u32::from_be_bytes(file[8..12].try_into().unwrap()),
            CFE_FS_HEADER_BYTES as u32
        );

        let tbl_off = CFE_FS_HEADER_BYTES;
        assert_eq!(
            u32::from_be_bytes(file[tbl_off + 4..tbl_off + 8].try_into().unwrap()),
            0
        ); // Offset
        assert_eq!(
            u32::from_be_bytes(file[tbl_off + 8..tbl_off + 12].try_into().unwrap()),
            raw.len() as u32
        );

        let image_off = CFE_TBL_COMBINED_HDR_BYTES;
        assert_eq!(
            u64::from_le_bytes(file[image_off..image_off + 8].try_into().unwrap()),
            AI_APP_WEIGHTS_TBL_MAGIC
        );
        assert_eq!(&file[image_off..], raw.as_slice());
    }
}
