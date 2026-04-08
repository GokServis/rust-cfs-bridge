//! Build a byte-perfect `AI_APP_WeightsTable_t` table image for cFE Table Services.
//!
//! This matches the layout in `cfs/apps/ai_app/fsw/inc/ai_app_tbl.h` and the CRC rules in
//! `cfs/apps/ai_app/fsw/src/ai_app_tbl_mgr.c`:
//! - CRC32 is IEEE reflected (init 0xFFFFFFFF, xorout 0xFFFFFFFF)
//! - computed over the entire table image bytes while skipping the 4 bytes of `Hdr.Crc32`.

use std::fmt;

pub const AI_APP_WEIGHTS_TBL_MAGIC: u64 = 0x4149_5F41_5050_5F57u64; // "AI_APP_W"
pub const AI_APP_WEIGHTS_TBL_VERSION: u32 = 1;
pub const AI_APP_WEIGHTS_TBL_MISSION_VER_LEN: usize = 64;

/// Bytes occupied by `AI_APP_WeightsTblHdr_t` in the C layout (gcc/clang, typical embedded ABI).
///
/// Logical header fields are 100 bytes; the compiler inserts **4 bytes** tail padding so `double`
/// table data matches `offsetof(AI_APP_WeightsTable_t, wte) == 104`. The CRC is computed over the
/// full `sizeof(AI_APP_WeightsTable_t)` image, including this padding and any trailing struct padding.
pub const AI_APP_WEIGHTS_TBL_HDR_LAYOUT_BYTES: usize = 104;

/// Byte offset of `Hdr.Crc32` within the table image for the lab target layout.
///
/// Per the C header order: Magic(u64) at 0..8, Version(u32) at 8..12, then Crc32(u32) at 12..16.
pub const AI_APP_WEIGHTS_TBL_CRC32_OFF: usize = 12;
pub const AI_APP_WEIGHTS_TBL_CRC32_LEN: usize = 4;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AiAppDims {
    pub vocab_size: u32,
    pub n_embd: u32,
    pub block_size: u32,
    pub n_head: u32,
    /// Number of layers to use (must be 1..=max_layer).
    pub n_layer: u32,
    /// Upper bound for static arrays (matches `AI_APP_GPT_MAX_LAYER` on flight).
    pub max_layer: u32,
    /// MLP factor (hidden = n_embd * mlp_factor).
    pub mlp_factor: u32,
}

impl AiAppDims {
    pub fn mlp_hidden(&self) -> u32 {
        self.n_embd
            .checked_mul(self.mlp_factor)
            .expect("mlp_hidden overflow")
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct AiAppWeights<'a> {
    pub wte: &'a [f64],
    pub wpe: &'a [f64],
    pub lm_head: &'a [f64],
    /// Per-layer weights; each inner slice must be length n_embd*n_embd (or derived).
    pub attn_wq: &'a [&'a [f64]],
    pub attn_wk: &'a [&'a [f64]],
    pub attn_wv: &'a [&'a [f64]],
    pub attn_wo: &'a [&'a [f64]],
    pub mlp_fc1: &'a [&'a [f64]],
    pub mlp_fc2: &'a [&'a [f64]],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TableImageError {
    BadLen {
        field: &'static str,
        expected: usize,
        got: usize,
    },
    BadLayerCount {
        n_layer: u32,
        max_layer: u32,
    },
}

impl fmt::Display for TableImageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TableImageError::BadLen {
                field,
                expected,
                got,
            } => {
                write!(f, "{field} len mismatch: expected {expected}, got {got}")
            }
            TableImageError::BadLayerCount { n_layer, max_layer } => {
                write!(f, "bad n_layer {n_layer} (max_layer {max_layer})")
            }
        }
    }
}

impl std::error::Error for TableImageError {}

fn push_u32_le(buf: &mut Vec<u8>, v: u32) {
    buf.extend_from_slice(&v.to_le_bytes());
}
fn push_u64_le(buf: &mut Vec<u8>, v: u64) {
    buf.extend_from_slice(&v.to_le_bytes());
}
fn push_f64_le(buf: &mut Vec<u8>, v: f64) {
    buf.extend_from_slice(&v.to_le_bytes());
}
fn push_fixed_cstr(buf: &mut Vec<u8>, s: &str, n: usize) {
    let b = s.as_bytes();
    let take = b.len().min(n);
    buf.extend_from_slice(&b[..take]);
    buf.resize(buf.len() + (n - take), 0);
}

fn expect_len(field: &'static str, got: usize, expected: usize) -> Result<(), TableImageError> {
    if got == expected {
        Ok(())
    } else {
        Err(TableImageError::BadLen {
            field,
            expected,
            got,
        })
    }
}

/// Compute CRC32 update step (reflected) identical to `ai_app_tbl_mgr.c`.
fn crc32_update(mut crc: u32, byte: u8) -> u32 {
    crc ^= byte as u32;
    for _ in 0..8 {
        if (crc & 1) != 0 {
            crc = (crc >> 1) ^ 0xEDB8_8320;
        } else {
            crc >>= 1;
        }
    }
    crc
}

/// Computes the `ai_app` CRC32 over `data` skipping the CRC32 field bytes.
pub fn ai_app_crc32_skip_field(data: &[u8], crc_off: usize) -> u32 {
    let mut crc = 0xFFFF_FFFFu32;
    for (i, &b) in data.iter().enumerate() {
        if i >= crc_off && i < crc_off + AI_APP_WEIGHTS_TBL_CRC32_LEN {
            continue;
        }
        crc = crc32_update(crc, b);
    }
    crc ^ 0xFFFF_FFFFu32
}

/// Builds the full table image and fills `Hdr.Crc32` (little-endian u32) in-place.
pub fn build_ai_app_weights_table_image(
    dims: &AiAppDims,
    mission_version: &str,
    weights: &AiAppWeights<'_>,
) -> Result<Vec<u8>, TableImageError> {
    if dims.n_layer == 0 || dims.n_layer > dims.max_layer {
        return Err(TableImageError::BadLayerCount {
            n_layer: dims.n_layer,
            max_layer: dims.max_layer,
        });
    }

    let vocab = dims.vocab_size as usize;
    let n_embd = dims.n_embd as usize;
    let block = dims.block_size as usize;
    let n_layer = dims.n_layer as usize;
    let max_layer = dims.max_layer as usize;
    let mlp_hidden = dims.mlp_hidden() as usize;

    let wte_len = vocab * n_embd;
    let wpe_len = block * n_embd;
    let lm_len = vocab * n_embd;
    expect_len("wte", weights.wte.len(), wte_len)?;
    expect_len("wpe", weights.wpe.len(), wpe_len)?;
    expect_len("lm_head", weights.lm_head.len(), lm_len)?;

    let attn_len = n_embd * n_embd;
    let mlp_fc1_len = mlp_hidden * n_embd;
    let mlp_fc2_len = n_embd * mlp_hidden;

    // Per-layer arrays are sized by max_layer in the C struct; only first n_layer are used.
    expect_len("attn_wq[layer_count]", weights.attn_wq.len(), max_layer)?;
    expect_len("attn_wk[layer_count]", weights.attn_wk.len(), max_layer)?;
    expect_len("attn_wv[layer_count]", weights.attn_wv.len(), max_layer)?;
    expect_len("attn_wo[layer_count]", weights.attn_wo.len(), max_layer)?;
    expect_len("mlp_fc1[layer_count]", weights.mlp_fc1.len(), max_layer)?;
    expect_len("mlp_fc2[layer_count]", weights.mlp_fc2.len(), max_layer)?;

    for li in 0..max_layer {
        expect_len("attn_wq[layer]", weights.attn_wq[li].len(), attn_len)?;
        expect_len("attn_wk[layer]", weights.attn_wk[li].len(), attn_len)?;
        expect_len("attn_wv[layer]", weights.attn_wv[li].len(), attn_len)?;
        expect_len("attn_wo[layer]", weights.attn_wo[li].len(), attn_len)?;
        expect_len("mlp_fc1[layer]", weights.mlp_fc1[li].len(), mlp_fc1_len)?;
        expect_len("mlp_fc2[layer]", weights.mlp_fc2[li].len(), mlp_fc2_len)?;
    }

    // Serialize header
    let mut buf: Vec<u8> = Vec::new();
    push_u64_le(&mut buf, AI_APP_WEIGHTS_TBL_MAGIC);
    push_u32_le(&mut buf, AI_APP_WEIGHTS_TBL_VERSION);
    push_u32_le(&mut buf, 0); // placeholder CRC32
    push_u32_le(&mut buf, dims.vocab_size);
    push_u32_le(&mut buf, dims.n_embd);
    push_u32_le(&mut buf, dims.block_size);
    push_u32_le(&mut buf, dims.n_head);
    push_u32_le(&mut buf, dims.n_layer);
    push_fixed_cstr(
        &mut buf,
        mission_version,
        AI_APP_WEIGHTS_TBL_MISSION_VER_LEN,
    );
    debug_assert_eq!(buf.len(), 100);
    while buf.len() < AI_APP_WEIGHTS_TBL_HDR_LAYOUT_BYTES {
        buf.push(0);
    }

    // Serialize arrays (must match C struct order)
    for &x in weights.wte {
        push_f64_le(&mut buf, x);
    }
    for &x in weights.wpe {
        push_f64_le(&mut buf, x);
    }
    for &x in weights.lm_head {
        push_f64_le(&mut buf, x);
    }

    // Per-layer arrays are serialized for all max_layer entries, in declaration order.
    for li in 0..max_layer {
        for &x in weights.attn_wq[li] {
            push_f64_le(&mut buf, x);
        }
    }
    for li in 0..max_layer {
        for &x in weights.attn_wk[li] {
            push_f64_le(&mut buf, x);
        }
    }
    for li in 0..max_layer {
        for &x in weights.attn_wv[li] {
            push_f64_le(&mut buf, x);
        }
    }
    for li in 0..max_layer {
        for &x in weights.attn_wo[li] {
            push_f64_le(&mut buf, x);
        }
    }
    for li in 0..max_layer {
        for &x in weights.mlp_fc1[li] {
            push_f64_le(&mut buf, x);
        }
    }
    for li in 0..max_layer {
        for &x in weights.mlp_fc2[li] {
            push_f64_le(&mut buf, x);
        }
    }

    // Fill CRC32 field in-place
    let crc = ai_app_crc32_skip_field(&buf, AI_APP_WEIGHTS_TBL_CRC32_OFF);
    buf[AI_APP_WEIGHTS_TBL_CRC32_OFF..AI_APP_WEIGHTS_TBL_CRC32_OFF + AI_APP_WEIGHTS_TBL_CRC32_LEN]
        .copy_from_slice(&crc.to_le_bytes());

    // Basic sanity: ensure CRC is nonzero for non-default mission versions (flight will reject otherwise).
    // Keep this as a fail-fast guardrail.
    if crc == 0 && !mission_version.starts_with("DEFAULT_") {
        // This should be vanishingly rare; but if it happens, it is operationally invalid.
        // Reuse BadLen as a generic error is ugly; prefer a dedicated error if this ever trips.
        return Err(TableImageError::BadLen {
            field: "Hdr.Crc32",
            expected: 1,
            got: 0,
        });
    }

    // Ensure we didn't accidentally omit layers: we serialized max_layer, not n_layer.
    let _ = n_layer; // reserved for future invariants
    Ok(buf)
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let mlp_hidden = d.mlp_hidden() as usize;
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
    fn crc32_field_offset_is_12_bytes() {
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
        let img = build_ai_app_weights_table_image(&d, "MISSION_X", &w).unwrap();
        assert!(img.len() > AI_APP_WEIGHTS_TBL_CRC32_OFF + 4);
        // Magic is LE u64 at 0..8, Version LE u32 at 8..12. Therefore CRC field begins at 12.
        assert_eq!(AI_APP_WEIGHTS_TBL_CRC32_OFF, 12);
        // `sizeof(AI_APP_WeightsTable_t)` on lab gcc (see ai_app_tbl.h _Static_assert).
        assert_eq!(img.len(), 104808);
    }

    #[test]
    fn crc32_skip_field_detects_bitflip_outside_crc_field() {
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
        let mut img = build_ai_app_weights_table_image(&d, "MISSION_X", &w).unwrap();
        let orig_crc = u32::from_le_bytes(
            img[AI_APP_WEIGHTS_TBL_CRC32_OFF..AI_APP_WEIGHTS_TBL_CRC32_OFF + 4]
                .try_into()
                .unwrap(),
        );
        // flip a byte outside crc field
        img[AI_APP_WEIGHTS_TBL_CRC32_OFF + 4] ^= 0x01;
        let calc = ai_app_crc32_skip_field(&img, AI_APP_WEIGHTS_TBL_CRC32_OFF);
        assert_ne!(calc, orig_crc);
    }

    #[test]
    fn crc32_skip_field_ignores_bitflip_inside_crc_field() {
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
        let mut img = build_ai_app_weights_table_image(&d, "MISSION_X", &w).unwrap();
        let calc_before = ai_app_crc32_skip_field(&img, AI_APP_WEIGHTS_TBL_CRC32_OFF);
        // flip a byte *inside* crc field
        img[AI_APP_WEIGHTS_TBL_CRC32_OFF] ^= 0x80;
        let calc_after = ai_app_crc32_skip_field(&img, AI_APP_WEIGHTS_TBL_CRC32_OFF);
        assert_eq!(calc_after, calc_before);
    }
}
