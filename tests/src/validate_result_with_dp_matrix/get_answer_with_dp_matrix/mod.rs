use crate::{
    test_data_path::*,
    dynamic_programming_matrix::DpMatrix,
};
use log::info;
use sigalign::results::AlignmentResult;
use std::{path::PathBuf, ops::Range, io::{Read, Write}};

mod local_with_all_substring;
mod semi_global;

pub use local_with_all_substring::{
    get_cached_local_all_substring_to_pattern_matched_targets_result_with_dp_matrix,
};
pub use semi_global::{
    get_cached_semi_global_result_with_dp_matrix,
};

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