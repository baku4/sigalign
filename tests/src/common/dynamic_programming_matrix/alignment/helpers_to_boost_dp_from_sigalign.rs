use ahash::AHashSet;
use sigalign::{
    Aligner,
    algorithms::Local,
    Reference,
};
use sigalign_core::aligner::AlignmentRegulator;

pub fn calculate_the_pattern_size(
    mismatch_penalty: u32,
    gap_open_penalty: u32,
    gap_extend_penalty: u32,
    min_length: u32,
    max_penalty_per_length: f32,
) -> Option<u32> {
    let regulator = AlignmentRegulator::new(mismatch_penalty, gap_open_penalty, gap_extend_penalty, min_length, max_penalty_per_length).ok()?;
    Some(regulator.get_pattern_size())
}
pub fn target_indices_having_matched_pattern(
    query: &[u8],
    sig_reference: &Reference,
    pattern_size: u32,
) -> Vec<u32> {
    let sorted_target_indices: Vec<u32> = (0..sig_reference.as_ref().num_targets()).into_iter().collect();
    let mut target_index_set = AHashSet::new();
    let pattern_count = query.len() / pattern_size as usize;
    for pattern_index in 0..pattern_count {
        let start_index = pattern_size as usize * pattern_index;
        let last_index = start_index + pattern_size as usize;
        let pattern = &query[start_index..last_index];
        let pattern_location = sig_reference.as_ref().locate_pattern(
            pattern,
            &sorted_target_indices,
        );
        for v in pattern_location {
            target_index_set.insert(v.target_index);
        }
    }

    target_index_set.into_iter().collect()
}