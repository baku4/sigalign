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
    Divisible,
    LabelStorage,
    RcStorage,
    Serializable,
    SizeAware,
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
use test_data::{
    get_lf_fa_path,
    get_crlf_fa_path,
    get_two_line_fa_path,
    get_ref_for_val_path,
    get_qry_for_val_path,
    get_local_tmp_dir,
};

use log::{info, warn, error};
#[cfg(test)]
pub fn init_logger() {
    let _ = env_logger::builder()
        .target(env_logger::Target::Stdout)
        .is_test(true)
        .try_init();
}

// Test Main
// Fasta reader can read various type of FASTA formatted file
#[cfg(test)]
mod read_fasta;
// Sequence storages provide same information
#[cfg(test)]
mod sequence_storage;
#[cfg(test)]
mod reference_serialization;
#[cfg(test)]
mod same_result_with_dp;
// #[cfg(test)]
// mod print_alignment_result_to_cmp;
