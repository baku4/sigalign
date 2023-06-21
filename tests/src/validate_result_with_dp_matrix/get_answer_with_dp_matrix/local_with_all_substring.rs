use sigalign::wrapper::DefaultReference;

use super::*;
use crate::dynamic_programming_matrix::{
    local_all_substring_with_dpm_only_to_pattern_matched_targets,
};

pub fn get_cached_local_all_substring_to_pattern_matched_targets_result_with_dp_matrix(
    query: &[u8],
    label: &str,
    sig_reference: &DefaultReference,
    mismatch_penalty: u32,
    gap_open_penalty: u32,
    gap_extend_penalty: u32,
    min_length: u32,
    max_penalty_per_length: f32,
) -> Option<AlignmentResult> {
    let local_tmp_dir = get_local_tmp_dir().unwrap();
    let result_cache_name = format!(
        "DPM_LC_AS_PM_{}_{}_{}_{}_{}_{}_{}.json",
        label,
        "default_reference",
        mismatch_penalty,
        gap_open_penalty,
        gap_extend_penalty,
        min_length,
        (max_penalty_per_length * 1_000_000_f32) as u32,
    );
    let result_cache_file = local_tmp_dir.clone().join(result_cache_name);
    if !result_cache_file.exists() {
        info!("DPM generates new result");
            let result = local_all_substring_with_dpm_only_to_pattern_matched_targets(
                &query,
                sig_reference,
                mismatch_penalty,
                gap_open_penalty,
                gap_extend_penalty,
                min_length,
                max_penalty_per_length,
            );
            save_result_to_file(&result_cache_file, &result);
    } else {
        info!("DPM gets cached result");
    }
    Some(parse_cached_result(&result_cache_file))
}