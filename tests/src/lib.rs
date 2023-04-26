use anyhow::{Result, bail as error_msg};

// Test Data
pub mod test_data_path;
pub mod random_text_and_pattern;
// DP matrix to generate the answer result
mod dynamic_programming_matrix;

// Aligner to validate the result
mod dp_based_aligner;
pub use dp_based_aligner::DpBasedAligner;
// Logger
mod logger;
use logger::init_logger;

// Test Main
mod readme_example;
// Validate result
mod validate_result;
pub use validate_result::{
    get_sample_result_of_val_test,
};
// Fasta reader can read various type of FASTA formatted file
mod read_fasta;
// Pattern Index
mod pattern_index;
// Test sequence storages
// mod sequence_storage;
// Save and load reference
mod reference_serialization;
// Compare result with "Dynamic Programming" method
mod same_result_with_dp;
// #[cfg(test)]
// mod print_alignment_result_to_cmp;
