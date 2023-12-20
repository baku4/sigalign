/*!
# SigAlign

SigAlign is a library for gap-affine sequence alignment tasks guided by explicit similarity cutoffs.

## Quick Start
```rust
use sigalign::{Aligner, ReferenceBuilder};

// (1) Build `Reference`
let fasta =
br#">target_1
ACACAGATCGCAAACTCACAATTGTATTTCTTTGCCACCTGGGCATATACTTTTTGCGCCCCCTCATTTA
>target_2
TCTGGGGCCATTGTATTTCTTTGCCAGCTGGGGCATATACTTTTTCCGCCCCCTCATTTACGCTCATCAC"#;
let reference = ReferenceBuilder::new()
    .ignore_case(true)
    .ignore_base(b'N')
    .add_fasta(&fasta[..]).unwrap()
    .build().unwrap();

// (2) Make `Aligner`
let mut aligner = Aligner::new(
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
```

## Core Structures

The core of SigAlign is built around two central structures: `Reference` and `Aligner`. The `Reference` serves as a database for multiple target sequences, while the `Aligner` is responsible for executing the alignment tasks. The basic workflow is as follows:

1. Instantiate `Reference` and `Aligner`
2. Pass the `Reference` and query sequence to the `Aligner`.

## Inputs and Outputs

### Inputs
- Sequences
    - Target sequences
    - Query sequence
- Regulators
    - Penalties
        - Mismatch penalty
        - Gap-open penalty
        - Gap-extend penalty
    - Cutoffs
        - Minimum alignment length (MinL)
        - Maximum penalty per alignment length (MaxP)
### Outputs
- Alignment Results
    - Penalty score
    - Length of alignment
    - Alignment position
    - Operations
*/

pub mod results;

mod reference;
pub use reference::{
    Reference,
    ReferenceBuilder,
    ReferenceBuildError,
    ReferenceLoadError,
};

mod aligner;
pub use aligner::{
    Aligner,
    AlignerBuildError,
};
