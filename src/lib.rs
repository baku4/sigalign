/*!
# sigalign
Similarity Guided Alignment Algorithm
---
# Quick Start
```rust
use sigalign::{Reference, Aligner};
use sigalign::reference::InMemoryProvider;

// (1) Make `Reference`
let mut sequence_provider = InMemoryProvider::new_empty();
sequence_provider.add_labeled_sequence(
    "record_1".to_string(),
    b"ATCAAACTCACAATTGTATTTCTTTGCCAGCTGGGCATATACTTTTTCCGCACCCTCATTTAACTTCTTGGATAACGGAAGCACACCGATCTTAACCGGAGCAAGTGCCGGATGAAAATGGAAAACGGTTCTTACGTCCGGCTTTTCCTCTGTTCCGATATTTTCCTCATCGTATGCAGCACATAAAAATGCCAGAACCA".to_vec(),
);
sequence_provider.add_labeled_sequence(
    "record_2".to_string(),
    b"TTCCATCAAACTCACAATTGTATTTCTTTGCCAGCTGGGCATATACTTTTTCCGCACCCTCATTTAACTTCTTGGATAACGGAAGCACACCGATCTTAACCGGAGCGTATGCAGCACATAAAAAT".to_vec(),
);
let mut reference = Reference::new_with_default_config(sequence_provider).unwrap();

// (2) Make `Aligner`
let aligner = Aligner::new(4, 6, 2, 100, 0.1).unwrap();

// (3) Alignment with query
let query = b"TTCCTCTGTCATCAAACTCACAATTGTATTTCTTTGCCAGCTGGGCATATACTTTTTCCGCCCCCTCATTTAACTTCTTGGATAACGGAAGCACACCGATCTTAACCGGAGGTGCCGGATGAAAATGGAAAACGGTTCTTACGTCCGGCTTTTCCTCTGTTCCGATATTTTCCTCAT";
// - Semi-global alignment
let result_semi_global: String = aligner.semi_global_alignment_labeled(&mut reference, query).unwrap();
// - Local alignment
let result_local: String = aligner.local_alignment_labeled(&mut reference, query).unwrap();
```
*/

use anyhow::{Result, bail as error_msg};
use serde::{Serialize, Deserialize, de::DeserializeOwned};

#[doc(hidden)]
// Core
mod core;
#[doc(hidden)]
// Algorithm
mod algorithm;
pub mod reference;
pub mod aligner;
#[doc(hidden)]
pub mod deprecated;
#[cfg(test)]
mod tests;

pub use reference::Reference;
pub use aligner::Aligner;

mod example {
    #[test]
    fn print_library_example_quick_start() {
        use crate::{Reference, Aligner};
        use crate::reference::InMemoryProvider;

        // (1) Make `Reference`
        let mut sequence_provider = InMemoryProvider::new_empty();
        sequence_provider.add_labeled_sequence(
            "record_1".to_string(),
            b"ATCAAACTCACAATTGTATTTCTTTGCCAGCTGGGCATATACTTTTTCCGCACCCTCATTTAACTTCTTGGATAACGGAAGCACACCGATCTTAACCGGAGCAAGTGCCGGATGAAAATGGAAAACGGTTCTTACGTCCGGCTTTTCCTCTGTTCCGATATTTTCCTCATCGTATGCAGCACATAAAAATGCCAGAACCA".to_vec(),
        );
        sequence_provider.add_labeled_sequence(
            "record_2".to_string(),
            b"TTCCATCAAACTCACAATTGTATTTCTTTGCCAGCTGGGCATATACTTTTTCCGCACCCTCATTTAACTTCTTGGATAACGGAAGCACACCGATCTTAACCGGAGCGTATGCAGCACATAAAAAT".to_vec(),
        );
        let mut reference = Reference::new_with_default_config(sequence_provider).unwrap();

        // (2) Make `Aligner`
        let aligner = Aligner::new(4, 6, 2, 100, 0.1).unwrap();

        // (3) Alignment with query
        let query = b"TTCCTCTGTCATCAAACTCACAATTGTATTTCTTTGCCAGCTGGGCATATACTTTTTCCGCCCCCTCATTTAACTTCTTGGATAACGGAAGCACACCGATCTTAACCGGAGGTGCCGGATGAAAATGGAAAACGGTTCTTACGTCCGGCTTTTCCTCTGTTCCGATATTTTCCTCAT";
        // - Semi-global alignment
        let result_semi_global: String = aligner.semi_global_alignment_labeled(&mut reference, query).unwrap();
        // - Local alignment
        let result_local: String = aligner.local_alignment_labeled(&mut reference, query).unwrap();

        println!("aligner:\n{:#?}", aligner);

        println!("{:#?}", result_semi_global);
        println!("{:#?}", result_local);
    }
}