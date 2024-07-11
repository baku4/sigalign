use super::{
    DpMatrix,
    parse_valid_local_result_from_dpm,
    parse_valid_semi_global_result_from_dpm,
};
use sigalign::results::{
    QueryAlignment,
    TargetAlignment,
    Alignment,
    AlignmentOperation,
};
use std::path::PathBuf;
use ahash::AHashSet;

mod helpers_to_boost_dp_using_pattern_of_sigalign;
use helpers_to_boost_dp_using_pattern_of_sigalign::{
    target_indices_having_matched_pattern,
};

mod semi_global;
pub use semi_global::{
    dp_semi_global_to_pattern_existing_targets,
    dp_semi_global_to_ref_file,
    dp_semi_global_to_target,
};

mod local_with_one_matrix;
pub use local_with_one_matrix::{
    dp_local_to_to_pattern_existing_targets,
    dp_local_to_ref_file,
    dp_local_to_target,
};

// mod local_with_all_substring;
