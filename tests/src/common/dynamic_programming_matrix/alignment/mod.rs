use super::DpMatrix;
use sigalign::results::{
    QueryAlignment,
    TargetAlignment,
    Alignment,
    AlignmentOperation,
};
use std::path::PathBuf;
use ahash::AHashSet;

mod helpers_to_boost_dp_from_sigalign;
use helpers_to_boost_dp_from_sigalign::{
    calculate_the_pattern_size,
    target_indices_having_matched_pattern,
};

mod semi_global;
mod local_with_all_substring;
mod local_with_one_matrix;

pub use local_with_all_substring::{
    local_all_substring_with_dpm_only_to_pattern_matched_targets,
};
pub use semi_global::{
    semi_global_with_dpm,
};
