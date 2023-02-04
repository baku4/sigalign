use sigalign::{
    Aligner,
    Reference,
    ReferenceBuilder,
};
use sigalign::core::*;
use sigalign::util::*;
use sigalign::result::*;
use sigalign::sequence_storage::{
    // Trait
    SequenceStorage,
    Divide,
    LabelStorage,
    RcStorage,
    Serialize,
    EstimateSize,
    // Storage
    InMemoryStorage,
    InMemoryRcStorage,
    IndexedFastaStorage,
    IndexedFastaRcStorage,
};
use anyhow::{Result, bail as error_msg};
use ahash::{AHashMap, AHashSet};

// Aligner to verifying result
mod dp_based_aligner;
use dp_based_aligner::DpBasedAligner;

// Data Path
pub mod test_data;
pub mod random_text;

use log::{info, warn, error};
pub fn init_logger() {
    let _ = env_logger::builder()
        .target(env_logger::Target::Stdout)
        .is_test(true)
        .try_init();
}

// Test Main
// Validate result
mod validate_result; 
// Fasta reader can read various type of FASTA formatted file
mod read_fasta;
// Test sequence storages
mod sequence_storage;
// Save and load reference
mod reference_serialization;
// Compare result with "Dynamic Programming" method
mod same_result_with_dp;
// #[cfg(test)]
// mod print_alignment_result_to_cmp;
