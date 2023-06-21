#[test]
fn quick_start() {
    
use sigalign::wrapper::{
    DefaultAligner,
    DefaultReference,
};

// (1) Build `Reference`
let fasta =
br#">record_1
ACACAGATCGCAAACTCACAATTGTATTTCTTTGCCACCTGGGCATATACTTTTTGCGCCCCCTCATTTA
>record_2
TCTGGGGCCATTGTATTTCTTTGCCAGCTGGGGCATATACTTTTTCCGCCCCCTCATTTACGCTCATCAC"#;
let reference = DefaultReference::from_fasta_bytes(fasta).unwrap();

// (2) Make `Aligner`
let mut aligner = DefaultAligner::new(
    4,   // Mismatch penalty
    6,   // Gap-open penalty
    2,   // Gap-extend penalty
    50,  // Minimum aligned length
    0.2, // Maximum penalty per length
).unwrap();

// (3) Align query to reference
let query = b"CAAACTCACAATTGTATTTCTTTGCCAGCTGGGCATATACTTTTTCCGCCCCCTCATTTAACTTCTTGGA";
let result = aligner.align_query(&reference, query);
println!("{:#?}", result);

}