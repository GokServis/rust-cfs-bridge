//! Deterministic "golden" weight generator for the microgpt toy model.
//!
//! This is for end-to-end plumbing tests: stable bytes, stable CRC32, stable table load/activate.

use super::table_image::AiAppDims;

#[derive(Clone, Debug)]
pub struct GoldenWeightsOwned {
    pub wte: Vec<f64>,
    pub wpe: Vec<f64>,
    pub lm_head: Vec<f64>,
    pub attn_wq: Vec<Vec<f64>>,
    pub attn_wk: Vec<Vec<f64>>,
    pub attn_wv: Vec<Vec<f64>>,
    pub attn_wo: Vec<Vec<f64>>,
    pub mlp_fc1: Vec<Vec<f64>>,
    pub mlp_fc2: Vec<Vec<f64>>,
}

impl GoldenWeightsOwned {
    // Intentionally no `as_borrowed()` here: `AiAppWeights` borrows `&[&[f64]]`, which
    // must be backed by vectors that outlive the call site. Construct those at the call site.
}

#[derive(Clone, Debug)]
struct XorShift64 {
    x: u64,
}

impl XorShift64 {
    fn new(seed: u64) -> Self {
        Self { x: seed.max(1) }
    }
    fn next_u64(&mut self) -> u64 {
        // xorshift64*
        let mut x = self.x;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.x = x;
        x.wrapping_mul(0x2545F4914F6CDD1D)
    }
    fn next_f64(&mut self) -> f64 {
        // Uniform in [0,1), using top 53 bits
        let u = self.next_u64() >> 11;
        (u as f64) * (1.0 / ((1u64 << 53) as f64))
    }
    fn next_f64_signed(&mut self, scale: f64) -> f64 {
        // Uniform in [-scale, scale]
        (self.next_f64() * 2.0 - 1.0) * scale
    }
}

/// Deterministic microgpt toy weights for the given dims.
///
/// The generated values are finite and comfortably within `AI_APP_WT_MIN/MAX`.
pub fn generate_microgpt_golden(dims: &AiAppDims) -> GoldenWeightsOwned {
    let vocab = dims.vocab_size as usize;
    let n_embd = dims.n_embd as usize;
    let block = dims.block_size as usize;
    let max_layer = dims.max_layer as usize;
    let mlp_hidden = dims.mlp_hidden() as usize;

    let mut rng = XorShift64::new(0x00C0_FFEE_1234_5678);
    let scale = 0.02;

    let mut wte = Vec::with_capacity(vocab * n_embd);
    for _ in 0..(vocab * n_embd) {
        wte.push(rng.next_f64_signed(scale));
    }
    let mut wpe = Vec::with_capacity(block * n_embd);
    for _ in 0..(block * n_embd) {
        wpe.push(rng.next_f64_signed(scale));
    }
    let mut lm_head = Vec::with_capacity(vocab * n_embd);
    for _ in 0..(vocab * n_embd) {
        lm_head.push(rng.next_f64_signed(scale));
    }

    let attn_len = n_embd * n_embd;
    let mut mk_layers = |len: usize| -> Vec<Vec<f64>> {
        let mut out = Vec::with_capacity(max_layer);
        for _ in 0..max_layer {
            let mut v = Vec::with_capacity(len);
            for _ in 0..len {
                v.push(rng.next_f64_signed(scale));
            }
            out.push(v);
        }
        out
    };

    GoldenWeightsOwned {
        wte,
        wpe,
        lm_head,
        attn_wq: mk_layers(attn_len),
        attn_wk: mk_layers(attn_len),
        attn_wv: mk_layers(attn_len),
        attn_wo: mk_layers(attn_len),
        mlp_fc1: mk_layers(mlp_hidden * n_embd),
        mlp_fc2: mk_layers(n_embd * mlp_hidden),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai_app::table_image::AiAppWeights;
    use crate::ai_app::table_image::{build_ai_app_weights_table_image, AiAppDims};

    #[test]
    fn golden_is_deterministic_and_builds_image() {
        let dims = AiAppDims {
            vocab_size: 17,
            n_embd: 16,
            block_size: 16,
            n_head: 4,
            n_layer: 1,
            max_layer: 4,
            mlp_factor: 4,
        };
        let g1 = generate_microgpt_golden(&dims);
        let g2 = generate_microgpt_golden(&dims);
        assert_eq!(g1.wte, g2.wte);
        fn layer_slices(v: &[Vec<f64>]) -> Vec<&[f64]> {
            v.iter().map(|x| x.as_slice()).collect()
        }
        let attn_wq = layer_slices(&g1.attn_wq);
        let attn_wk = layer_slices(&g1.attn_wk);
        let attn_wv = layer_slices(&g1.attn_wv);
        let attn_wo = layer_slices(&g1.attn_wo);
        let mlp_fc1 = layer_slices(&g1.mlp_fc1);
        let mlp_fc2 = layer_slices(&g1.mlp_fc2);
        let w = AiAppWeights {
            wte: &g1.wte,
            wpe: &g1.wpe,
            lm_head: &g1.lm_head,
            attn_wq: &attn_wq,
            attn_wk: &attn_wk,
            attn_wv: &attn_wv,
            attn_wo: &attn_wo,
            mlp_fc1: &mlp_fc1,
            mlp_fc2: &mlp_fc2,
        };
        let img = build_ai_app_weights_table_image(&dims, "LAB_GOLDEN", &w).unwrap();
        assert!(!img.is_empty());
    }
}
