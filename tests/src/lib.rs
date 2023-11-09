use anyhow::{Result, bail as error_msg};

//
// Requirements
//
// Test Data
pub mod test_data_path;
pub mod random_text_and_pattern;
// DP matrix to generate the answer result (NEW)
pub mod dynamic_programming_matrix;
// Logger
mod logger;
use logger::init_logger;

//
// Test Main
//
mod readme_example;
mod validate_result_with_dp_matrix;
mod validate_result_with_stable_version;
pub use validate_result_with_stable_version::{
    get_stable_result_of_val_data,
};
mod fasta_reader;
mod pattern_index;
mod reference_serialization;
// Test sequence storages
// mod sequence_storage;
// Save and load reference
// Compare result with "Dynamic Programming" method
// mod same_result_with_dp;
// #[cfg(test)]
// mod print_alignment_result_to_cmp;

#[test]
fn test_errored_query() {
    use sigalign::reference::{
        Reference,
        sequence_storage::in_memory::InMemoryStorage,
        pattern_index::lfi::{DynamicLfi, DynamicLfiOption},
    };
    use sigalign::wrapper::DefaultAligner;

    let mut ss = InMemoryStorage::new();
    ss.add_target("label", b"GCTTTTCCAGCAGATCCACCCTGTCTACACTACCTGCCTGTCCAGCAGATCAACACTGTCTACACTACCTGCTTTTCCAGCAGATCCACACTGTCTACACTACCTGCTTTTCCAGCAGATCCACCCCGTCTACACTACCTGCCTGGCCAGCATATCCACCCTGTCTACACTACCTGCTTTTCCAGTAGATCTGCCCTATCTACAATACCTGCTTGTCCAGCAGAACCACCCTGTCTATACTACCTGCCTGTCCAGCAGAACCACCCTGTCTATACTACCTGCCTGTCCAGCAGATCCACCCTGTCTACACTACCTGCCTGGCCAGCAGATCCGCCCTGTCTATACTACCTGCCGCTCCAGCAGATCCACCCTGTCTACACTACCTGCCTGTCCAGCAGACCCGCCCTGTCTACACTACCTGCCTGGTCAGTATATCCACCCTATCTACACTACCTGCCTGGCCAGCATATCCGCCCTGTCTACACTACCTGCCAGCCCAGCAGATCCGCCCTGTCTACACTACCTGCCTGGCCAGTAGATCCACGCTATCTACACTACCTGCCTGGCCAGCAGATCCACCCTGTCAACACTACCTGCTTGTCCAGCAGGTCCACACTGTCTACACTACCTGCCTGTCCAGCAGGTGCACCCTATCTACACTACCTGACTGTCCAGCAGATCCACCCTGTCTACACTACCTGCCTGTCCAAAAGATCCACCCTGTCTATATTACCTGCCTATACAGCAGAACTACCCTGTCTACACTACCAGCCTCCCCAGCAGATCCACCCTGTCTATACTACCTGCCTGGCCAGTAGATGCATCCTGTCTTCACTACCTGCTTGTCCAGCAGGTCCACCATGTCTACACTGCCTGCCTGGCCAGCAGATCCACCCTGTCTACACTACCTGCCTGCAAAGCAGATCCACCCTGTCTACACTACCTGGCTGGCCAGTAGATCCACGCTATCTACACTACCTTCCTGTCCAGCAGATCCAAC");

    let option = DynamicLfiOption {
        suffix_array_sampling_ratio: 1,
        max_lookup_table_byte_size: 100_000,
    };

    let reference: Reference<DynamicLfi, InMemoryStorage> = Reference::new(ss, option).unwrap();

    let mut aligner = DefaultAligner::new_local(
        4,
        6,
        2,
        100,
        0.1,
    ).unwrap();
    
    let mut result = aligner.align_query(&reference, b"CTCTACACTACCTGCCTGGCCAGCAGATCCGCCCTGTCTATACTACCTGCCGCTCCTGCGGATCCACCCTGTCTACACTACCTGCCTGTCCAGCAGACCCGCCCTGTCTACACTACCTGCCTGGTCAGTATATCCACCCTATCTACACTACCTGCCTGGCCAGCATATCCGCCCTGTCTACACTACCTGCCAGCCCAGCA");

    println!("{:#?}", result);
}
