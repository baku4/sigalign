use crate::aligner::AlignmentRegulator;

/// The result is None, if pattern size is not feasible (too low)
pub fn calculate_max_pattern_size(
    mismatch_penalty: u32,
    gap_open_penalty: u32,
    gap_extend_penalty: u32,
    minimum_aligned_length: u32,
    maximum_penalty_per_length: f32,
) -> Option<u32> {
    let regulator = AlignmentRegulator::new(
        mismatch_penalty,
        gap_open_penalty,
        gap_extend_penalty,
        minimum_aligned_length,
        maximum_penalty_per_length,
    );

    match regulator {
        Ok(v) => Some(v.pattern_size),
        Err(_) => None,
    }
}
