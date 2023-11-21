use anyhow::{Result, bail as error_msg};

//
// Requirements
//
// Test Data
pub mod test_data_path;
pub mod random_text_and_pattern;
// DP matrix to generate the answer result (NEW)
// pub mod dynamic_programming_matrix;
// Logger
mod logger;
use logger::init_logger;

//
// Test Main
//
// mod readme_example;
// mod validate_result_with_dp_matrix;
// mod validate_result_with_stable_version;
// pub use validate_result_with_stable_version::{
//     get_stable_result_of_val_data,
// };
// mod fasta_reader;
mod pattern_index;
// mod reference_serialization;
// Test sequence storages
// mod sequence_storage;
// Save and load reference
// Compare result with "Dynamic Programming" method
// mod same_result_with_dp;
// #[cfg(test)]
// mod print_alignment_result_to_cmp;
