#[test]
fn quick_start() {
    
use sigalign::{Aligner, Reference};

// (1) Build `Reference`
let fasta =
br#">target_1
ACACAGATCGCAAACTCACAATTGTATTTCTTTGCCACCTGGGCATATACTTTTTGCGCCCCCTCATTTA
>target_2
TCTGGGGCCATTGTATTTCTTTGCCAGCTGGGGCATATACTTTTTCCGCCCCCTCATTTACGCTCATCAC"#;
let reference = Reference::from_fasta(&fasta[..]).unwrap();

// (2) Make `Aligner`
let mut aligner = Aligner::new(
    4,   // Mismatch penalty
    6,   // Gap-open penalty
    2,   // Gap-extend penalty
    50,  // Minimum aligned length
    0.2, // Maximum penalty per length
    true, // is local alignment (false for semi-global alignment)
    None, // maximum number of alignments per query (None for unlimited)
).unwrap();

// (3) Align query to reference
let query = b"CAAACTCACAATTGTATTTCTTTGCCAGCTGGGCATATACTTTTTCCGCCCCCTCATTTAACTTCTTGGA";
let result = aligner.align_query(&reference, query);
println!("{:#?}", result);

}