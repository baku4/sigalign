# *SigAlign*
***Si**milarity-**g**uided **Align***
1. Have only **five** simplified parameters
    - Gap-affine penalties (3)
        1. Mismatch penalty
        2. Gap-open penalty
        3. Gap-extend penalty
    - Similarity cut-offs (2)
        1. Minimum length
        2. Maximum penalty per length
2. Give non-exceptional result
3. Reproducible by dynamic programming (DP) matrix

## Quick Start
```rust
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
let mut aligner = DefaultAligner::new_local(
    4,   // Mismatch penalty
    6,   // Gap-open penalty
    2,   // Gap-extend penalty
    50,  // Minimum aligned length
    0.2, // Maximum penalty per length
).unwrap();

// (3) Align query to reference
let query = b"CAAACTCACAATTGTATTTCTTTGCCAGCTGGGCATATACTTTTTCCGCCCCCTCATTTAACTTCTTGGA";
let result = aligner.align_query(&reference, query).unwrap();
println!("{}", result.to_json());
```
