#[test]
fn mod_aligner() {

use sigalign::wrapper::DefaultReference;
use sigalign::results::{
    AlignmentResult,
    fasta::FastaAlignmentResult,
};
use sigalign::aligner::{
    Aligner,
    mode::LocalMode,
    allocation_strategy::LinearStrategy,
};

let fasta =
br#">record_1
ACACAGATCGCAAACTCACAATTGTATTTCTTTGCCACCTGGGCATATACTTTTTGCGCCCCCTCATTTA
>record_2
TCTGGGGCCATTGTATTTCTTTGCCAGCTGGGGCATATACTTTTTCCGCCCCCTCATTTACGCTCATCAC"#;
let reference = DefaultReference::from_fasta_bytes(fasta).unwrap();

// Initialize `Aligner`

//  vvv need to be mutable
let mut aligner = Aligner::<LocalMode, LinearStrategy>::new(
    4,  //mismatch_penalty,
    6,  // gap_open_penalty,
    2, // gap_extend_penalty,
    50, // minimum_aligned_length,
    0.1, // maximum_penalty_per_length,
).unwrap();

// Using `SequenceBuffer` (Lowest level)
let mut sequence_buffer = reference.get_sequence_buffer();
for query in [b"AA...CC", b"GG...TT"] {
    aligner.alignment(
        &reference,
        &mut sequence_buffer,
        query,
    );
}

// Using built in methods
let result: AlignmentResult = aligner.align_query(
    &reference,
    b"AA..TT",
);
let result: FastaAlignmentResult = aligner.align_fasta_file(
    &reference,
    "FASTA_FILE_PATH",
).unwrap();

}