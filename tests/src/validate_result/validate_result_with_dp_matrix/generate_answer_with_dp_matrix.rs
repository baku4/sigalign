use crate::{
    test_data_path::*,
    dynamic_programming_matrix::DpMatrix,
};
use log::info;
use sigalign::results::AlignmentResult;
use std::{path::PathBuf, ops::Range, io::{Read, Write}};

pub fn get_cached_local_result_with_dp_matrix(
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
        "DPM_LC_{}_{}_{}_{}_{}_{}_{}.json",
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
            let result = DpMatrix::local_alignment_query(
                &query,
                ref_file,
                mismatch_penalty,
                gap_open_penalty,
                gap_extend_penalty,
                min_length,
                max_penalty_per_length,
            );
            println!("local result:\n{:#?}", result);
            panic!("TEST");
            save_result_to_file(&result_cache_file, &result);
    } else {
        info!("DPM gets cached result");
    }
    parse_cached_result(&result_cache_file)
}
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
            let result = DpMatrix::semi_global_alignment_query(
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
fn save_result_to_file(
    cache_file: &PathBuf,
    result: &AlignmentResult,
) {
    let mut out_file = std::fs::File::create(cache_file).unwrap();
    let json = result.to_json();
    let buf = json.as_bytes();
    out_file.write_all(buf).unwrap();
}

fn parse_cached_result(cache_file: &PathBuf) -> AlignmentResult {
    let mut out_file = std::fs::File::open(cache_file).unwrap();

    let mut json = String::new();
    out_file.read_to_string(&mut json).unwrap();
    let result = AlignmentResult::from_json(&json).unwrap();

    result
}