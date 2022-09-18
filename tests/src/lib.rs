use sigalign::{
    Aligner,
    Reference,
    ReferenceBuilder,
};
use sigalign::core::*;
use sigalign::util::*;
use sigalign::result::*;
use sigalign::sequence_provider::{
    // Trait
    SequenceProvider,
    Divisible,
    LabelProvider,
    ReverseComplement,
    Serializable,
    SizeAware,
    // Provider
    InMemoryProvider,
    InMemoryRcProvider,
    IndexedFastaProvider,
    IndexedFastaRcProvider,
};
use anyhow::{Result, bail as error_msg};

// Aligner to verifying result
mod dp_based_aligner;
use dp_based_aligner::*;

// Data Path
pub mod test_data;
use test_data::{
    get_lf_fa_path,
    get_crlf_fa_path,
    get_two_line_fa_path,
    get_ref_for_val_path,
    get_qry_for_val_path,
};

// Test Main
// Fasta reader can read various type of FASTA formatted file
#[cfg(test)]
mod read_fasta;
// Sequence providers provide same information
#[cfg(test)]
mod sequence_provider;
#[cfg(test)]
mod reference_serialization;
// #[cfg(test)]
// mod print_alignment_result_to_cmp;

