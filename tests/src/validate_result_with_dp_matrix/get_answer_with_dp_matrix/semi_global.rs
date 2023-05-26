use super::*;
use crate::dynamic_programming_matrix::semi_global_with_dpm;

pub fn get_cached_semi_global_result_with_dp_matrix(
    query: &[u8],
    label: &str,
    ref_file: &PathBuf,
    mismatch_penalty: u32,
    gap_open_penalty: u32,
    gap_extend_penalty: u32,
    min_length: u32,
    max_penalty_per_length: f32,
) -> AlignmentResult {
    let local_tmp_dir = get_local_tmp_dir().unwrap();
    let result_cache_name = format!(
        "DPM_SG_{}_{}_{}_{}_{}_{}_{}.json",
        label,
        ref_file.file_stem().unwrap().to_str().unwrap(),
        mismatch_penalty,
        gap_open_penalty,
        gap_extend_penalty,
        min_length,
        (max_penalty_per_length * 1_000_000_f32) as u32,
    );
    let result_cache_file = local_tmp_dir.clone().join(result_cache_name);
    if !result_cache_file.exists() {
        info!("DPM generates new result");
            let result = semi_global_with_dpm(
                &query,
                ref_file,
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
    parse_cached_result(&result_cache_file)
}