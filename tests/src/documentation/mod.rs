#[test]
fn quick_start_example() {
    
use sigalign::{
    ReferenceBuilder,
    Aligner,
    algorithms::Local,
};

// (1) Build `Reference`
let fasta =
br#">target_1
ACACAGATCGCAAACTCACAATTGTATTTCTTTGCCACCTGGGCATATACTTTTTGCGCCCCCTCATTTA
>target_2
TCTGGGGCCATTGTATTTCTTTGCCAGCTGGGGCATATACTTTTTCCGCCCCCTCATTTACGCTCATCAC"#;
let reference = ReferenceBuilder::new()
    .set_uppercase(true)
    .ignore_base(b'N')
    .add_fasta(&fasta[..]).unwrap()
    .build().unwrap();

// (2) Initialize `Aligner`
let algorithm = Local::new(
    4,   // Mismatch penalty
    6,   // Gap-open penalty
    2,   // Gap-extend penalty
    50,  // Minimum aligned length
    0.2, // Maximum penalty per length
).unwrap();
let mut aligner = Aligner::new(algorithm);

// (3) Align query to reference
let query = b"CAAACTCACAATTGTATTTCTTTGCCAGCTGGGCATATACTTTTTCCGCCCCCTCATTTAACTTCTTGGA";
let result = aligner.align(query, &reference);
println!("{:#?}", result);

}