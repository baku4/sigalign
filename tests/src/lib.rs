use anyhow::{Result, bail as error_msg};

// Test Data
pub mod test_data_path;
pub mod random_text_and_pattern;
// DP matrix to generate the answer result
mod dynamic_programming_matrix;

// Aligner to validate the result
mod dp_based_aligner;
use dp_based_aligner::DpBasedAligner;
// Logger
mod logger;
use logger::init_logger;

// Test Main
// Validate result
mod validate_result;
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


#[test]
fn print_pattern_size() {
    use sigalign::wrapper::DefaultAligner;

    let aligner = DefaultAligner::new_local(
        4,
        6,
        2,
        u16::MAX as u32,
        0.0001,
    ).unwrap();

    print!("pattern_size: {}", aligner.get_pattern_size());
}