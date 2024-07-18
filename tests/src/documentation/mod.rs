#[test]
fn quick_start_example() {
    
use sigalign::{
    Aligner,
    algorithms::Local,
    ReferenceBuilder,
};

// (1) Build `Reference`
let fasta =
br#">target_1
ACACAGATCGCAAACTCACAATTGTATTTCTTTGCCACCTGGGCATATACTTTTTGCGCCCCCTCATTTA
>target_2
TCTGGGGCCATTGTATTTCTTTGCCAGCTGGGGCATATACTTTTTCCGCCCCCTCATTTACGCTCATCAC"#;
let reference = ReferenceBuilder::new()
    .set_uppercase(true) // Ignore case
    .ignore_base(b'N') // 'N' is never matched
    .add_fasta(&fasta[..]).unwrap() // Add sequences from FASTA
    .add_target(
        "target_3",
        b"AAAAAAAAAAA",
    ) // Add sequence manually
    .build().unwrap();

// (2) Initialize `Aligner`
let algorithm = Local::new(
    4,   // Mismatch penalty
    6,   // Gap-open penalty
    2,   // Gap-extend penalty
    50,  // Minimum length
    0.2, // Maximum penalty per length
).unwrap();
let mut aligner = Aligner::new(algorithm);

// (3) Align query to reference
let query = b"CAAACTCACAATTGTATTTCTTTGCCAGCTGGGCATATACTTTTTCCGCCCCCTCATTTAACTTCTTGGA";
let result = aligner.align(query, &reference);
println!("{:#?}", result);

}