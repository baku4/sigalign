use sigalign::result::{
    FastaAlignmentResult,
    FastaAlignmentLabeledResult,
};
use sigalign::{Aligner, Reference, ReferenceBuilder};
use sigalign::sequence_storage::{InMemoryStorage, InMemoryRcStorage};

const NUCLEOTIDE_ONLY_FA_PATH_1: &str = "../sample_data/nucleotide_only/ERR209055.fa";
const NUCLEOTIDE_ONLY_FA_PATH_2: &str = "../sample_data/nucleotide_only/ERR209056.fa";

pub fn get_test_alignment_result() -> FastaAlignmentLabeledResult {
    let mut aligner = get_test_local_aligner();
    let reference = get_test_reference();

    aligner.fasta_file_labeled_alignment(&reference, NUCLEOTIDE_ONLY_FA_PATH_2).unwrap()
}

fn get_test_local_aligner() -> Aligner {
    Aligner::new_local(5, 6, 3, 50, 0.05).unwrap()
}

fn get_test_reference() -> Reference<InMemoryRcStorage> {
    let mut sequence_storage = InMemoryRcStorage::new();
    sequence_storage.add_fasta_file(NUCLEOTIDE_ONLY_FA_PATH_1).unwrap();

    ReferenceBuilder::new().build(sequence_storage).unwrap()
}