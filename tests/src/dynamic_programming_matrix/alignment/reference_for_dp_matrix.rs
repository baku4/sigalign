use ahash::AHashSet;
use sigalign::{wrapper::DefaultReference, reference::ReferenceInterface};

pub fn target_indices_having_matched_pattern(
    query: &[u8],
    sig_reference: &DefaultReference,
    pattern_size: u32,
) -> Vec<u32> {
    let mut target_index_set = AHashSet::new();
    let pattern_count = query.len() / pattern_size as usize;
    for pattern_index in 0..pattern_count {
        let start_index = pattern_size as usize * pattern_index;
        let last_index = start_index + pattern_size as usize;
        let pattern = &query[start_index..last_index];
        let pattern_location = sig_reference.locate(pattern);
        for v in pattern_location {
            target_index_set.insert(v.target_index);
        }
    }

    target_index_set.into_iter().collect()
}