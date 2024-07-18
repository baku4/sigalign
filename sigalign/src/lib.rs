/*!
# SigAlign

SigAlign is a library for gap-affine sequence alignment tasks guided by explicit similarity cutoffs.

## Quick Start
```rust
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
```

## Core Structures
- `Reference`: A **database** for multiple target sequences.
   - **Generated** from `ReferenceBuilder`.
   - **Purpose**: Combining multiple sequences into one struct, indexing them to facilitate alignment processes.
   - Can be **immutable** while alignment.
- `Aligner`: An **executor** for alignment tasks.
   - **Generated** from `Algorithm`.
   - **Purpose**: Managing the workspace for alignment tasks.
   - Need to be **mutable** while alignment.

## Parameters: Definition of alignment results
- Penalties
    - Mismatch penalty (`u32`)
    - Gap-open penalty (`u32`)
    - Gap-extend penalty (`u32`)
- Cutoffs
    - Minimum alignment length (MinL) (`u32`)
    - Maximum penalty per alignment length (MaxP) (`f32`)

## Inputs and Outputs
- Inputs
    - Query: `&[u8]` (byte array)
    - Reference: ref(&) to `Reference`
- Outputs
    - `QueryAlignment`: A vector of `TargetAlignment` for each target sequence.
        - `TargetAlignment`: A vector of `Alignment` for each alignment.
            - Index: Index of the target sequence in Reference.
            - Alignment: Alignment results.
                - Penalty score
                - Length of alignment
                - Alignment position
                - Operations (Match, Substitution, Insertion, Deletion)
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
    algorithms,
};

pub mod utils;


#[cfg(test)]
mod doc_tests {
#[test]
fn test_readme() {


use crate::{
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
}
