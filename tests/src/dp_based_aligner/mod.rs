use super::{
    Aligner,
    PRECISION_SCALE,
};

use lt_fm_index::{LtFmIndex, LtFmIndexBuilder};

#[derive(Debug)]
struct DpBasedAligner {
    mismatch_penalty: usize,
    gap_open_penalty: usize,
    gap_extend_penalty: usize,
    minimum_aligned_length: usize,
    maximum_penalty_per_scale: usize,
    pattern_size: usize,
}

impl DpBasedAligner {
    pub fn new(
        mismatch_penalty: usize,
        gap_open_penalty: usize,
        gap_extend_penalty: usize,
        minimum_aligned_length: usize,
        maximum_penalty_per_length: f32,
    ) -> Self {
        let aligner = Aligner::new_local(mismatch_penalty, gap_open_penalty, gap_extend_penalty, minimum_aligned_length, maximum_penalty_per_length).unwrap();
        let pattern_size = aligner.get_pattern_size();

        let maximum_penalty_per_scale = (PRECISION_SCALE as f32 * maximum_penalty_per_length) as usize;

        Self {
            mismatch_penalty,
            gap_open_penalty,
            gap_extend_penalty,
            minimum_aligned_length,
            maximum_penalty_per_scale,
            pattern_size,
        }
    }
}
