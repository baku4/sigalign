use crate::{
    init_logger,
    test_data_path::*,
    dynamic_programming_matrix::DpMatrix,
};
use log::{info, error, warn};
use std::{path::PathBuf, ops::Range, io::{Read, Write}};
use ahash::{AHashMap, AHashSet};

pub fn get_answer_or_generate_semi_global_result_with_dp_matrix(
    qry_file: &PathBuf,
    ref_file: &PathBuf,
    qry_count: usize,
    mismatch_penalty: u32,
    gap_open_penalty: u32,
    gap_extend_penalty: u32,
    min_length: u32,
    max_penalty_per_length: f32,
) {
    // let mut qry_reader = FastaReader::from_path(qry_file).unwrap();
    // let mut ref_reader = FastaReader::from_path(qry_file).unwrap();
}
