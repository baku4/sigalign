use crate::{
    Result, error_msg,
    init_logger,
    test_data_path::*,
};
use ahash::AHashSet;
use sigalign::{
    wrapper::{
        DefaultAligner, DefaultReference,
    }, utils::FastaReader, results::{AlignmentResult, AnchorAlignmentResult},
};
use log::{info, error};
use super::{
    ALIGNER_OPTION,
    get_cached_semi_global_result_with_dp_matrix,
    get_cached_local_all_substring_to_pattern_matched_targets_result_with_dp_matrix,
};
use std::{
    sync::{mpsc, Arc, Mutex},
    path::PathBuf,
};
use std::thread;

mod semi_global;
use semi_global::generate_all_semi_global_answer_using_multiple_thread;
mod local;
use local::generate_all_local_answer_using_multiple_thread;

const THREAD_COUNT: usize = 8;

#[test]
fn generate_all_semi_global_answer() {
    generate_all_semi_global_answer_using_multiple_thread(THREAD_COUNT);
}
#[test]
fn generate_all_local_answer() {
    generate_all_local_answer_using_multiple_thread(THREAD_COUNT);
}
