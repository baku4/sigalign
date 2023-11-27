use super::DpMatrix;
use sigalign::{
    results::{
        AlignmentResult,
        TargetAlignmentResult, AnchorAlignmentResult, AlignmentOperation,
    },
    utils::FastaReader,
};
use std::path::PathBuf;
use ahash::AHashSet;

mod reference_for_dp_matrix;
use reference_for_dp_matrix::{
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
