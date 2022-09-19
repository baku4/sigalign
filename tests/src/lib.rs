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



#[test]
fn test_dp_based_alignment() {
    let ref_file = get_ref_for_val_path();
    let qry_file = get_qry_for_val_path();

    // Build reference
    let mut sequence_provider = InMemoryProvider::new();
    sequence_provider.add_fasta_file(ref_file).unwrap();

    let reference = ReferenceBuilder::new()
        .change_bwt_vector_size_to_128()
        .change_count_array_kmer(4).unwrap()
        .change_suffix_array_sampling_ratio(2).unwrap()
        .build(sequence_provider).unwrap();

    // Gen aligner
    let dp_based_aligner = DpBasedAligner::new(
        4,
        6,
        2,
        50,
        0.1,
    );

    println!("dp_based_aligner: {:?}", dp_based_aligner);

    // Do alignment
    let mut qry_reader = FastaReader::from_file_path(qry_file).unwrap();
    for _ in 0.. 4 {
        let (label, query) = qry_reader.next().unwrap();
        let alignment_result = dp_based_aligner.local_alignment(&reference, &query);

        println!("alignment_result: \n{:?}", alignment_result);
    }
}
