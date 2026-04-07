//! Default lab GPT dimensions — **must match**
//! `cfs/apps/ai_app/config/default_ai_app_mission_cfg.h` (`AI_APP_GPT_*` macros).
//!
//! If the C header changes, update these constants and golden generators together.

use super::table_image::AiAppDims;

pub const AI_APP_GPT_VOCAB_SIZE: u32 = 17;
pub const AI_APP_GPT_N_EMBD: u32 = 16;
pub const AI_APP_GPT_BLOCK_SIZE: u32 = 16;
pub const AI_APP_GPT_N_HEAD: u32 = 4;
pub const AI_APP_GPT_N_LAYER: u32 = 1;
pub const AI_APP_GPT_MAX_LAYER: u32 = 4;
pub const AI_APP_GPT_MLP_FACTOR: u32 = 4;

/// [`AiAppDims`] matching the mission default table size (`sizeof(AI_APP_WeightsTable_t)` on flight).
pub fn default_lab_ai_app_dims() -> AiAppDims {
    AiAppDims {
        vocab_size: AI_APP_GPT_VOCAB_SIZE,
        n_embd: AI_APP_GPT_N_EMBD,
        block_size: AI_APP_GPT_BLOCK_SIZE,
        n_head: AI_APP_GPT_N_HEAD,
        n_layer: AI_APP_GPT_N_LAYER,
        max_layer: AI_APP_GPT_MAX_LAYER,
        mlp_factor: AI_APP_GPT_MLP_FACTOR,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dims_match_c_header_comment_values() {
        let d = default_lab_ai_app_dims();
        assert_eq!(d.vocab_size, 17);
        assert_eq!(d.n_embd, 16);
        assert_eq!(d.block_size, 16);
        assert_eq!(d.n_head, 4);
        assert_eq!(d.n_layer, 1);
        assert_eq!(d.max_layer, 4);
        assert_eq!(d.mlp_factor, 4);
    }
}
