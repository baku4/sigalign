/*!
# SigAlign

SigAlign is a library designed to handle gap-affine pairwise sequence alignment tasks guided by easy-to-understand similarity cutoffs.

## Quick Start Example

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

// (2) Instantiate `Aligner`
let mut aligner = DefaultAligner::new(
    4,   // Mismatch penalty
    6,   // Gap-open penalty
    2,   // Gap-extend penalty
    50,  // Minimum aligned length
    0.2, // Maximum penalty per length
).unwrap();

// (3) Perform alignment
let query = b"CAAACTCACAATTGTATTTCTTTGCCAGCTGGGCATATACTTTTTCCGCCCCCTCATTTAACTTCTTGGA";
let result = aligner.align_query(&reference, query).unwrap();
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
        - Minimum length (ML)
        - Maximum penalty per length (MPL)
### Outputs
- Alignment Results
    - Penalty score
    - Length of alignment
    - Alignment position
    - Operation history

## Module Navigation
- [wrapper]: For ease of use, SigAlign provides a wrapper module which offers [DefaultReference](wrapper::DefaultReference) and [DefaultAligner](wrapper::DefaultAligner). These structures come with additional convenience methods.
- [reference](reference/index.html), [aligner]: Detailed implementations of [Aligner](aligner::Aligner) and [Reference](reference::Reference) are found in their respective modules: [reference](reference/index.html) and [aligner](aligner). Customizing each structure according to your usage requirements can help optimize performance.
*/

// Fundamentals
mod core;
mod algorithm;
pub mod results;

// Implementations
pub mod reference;
pub mod aligner;

// Utilities
pub mod wrapper;
pub mod utils;
