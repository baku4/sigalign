/*!
# sigalign
Similarity Guided Alignment Algorithm

## `Caveat`
This library is currently under development. Some key features are missing, and functions can be changed anytime without notification.

# What is Sigalign?
 - Sigalign is pairwise alignment algorithm using gap-affine penalty for nucleotide and amino-acid sequences.
## Features
 - Sigalign has two values that evaluate the similarity between sequences.
   - (1) Aligned length
   - (2) Alignment penalty per aligned length
 - Sigalign returns the result of sequences satisfying the similarity criterion above **without exception**.
 - Sigalign has **only five parameters** that affect the alignment result.
   - Three are penalty values.
     1. Mismatch penalty
     2. Gap open penalty
     3. Gap extend penalty
   - Two are cutoff for similarity values, which are aligned length and alignment penalty per aligned length.
     1. Minimum aligned length
     2. Maximum penalty per length
 - A `reference`, which is an alignment target, has two main features.
   - A search range can be specified after reference generation.
   - A method of storing sequence data can be selected in various ways depending on the purpose.
     - The `SequenceProvider` storing the sequence is defined as an interface (trait in `Rust`).
 - Sigalign provides two alignment algorithms
   - Semi-global alignment
     - Return the results of alignment that either the target or query sequence extended to the end from the left and right sides of the sequence respectively.
   - Local alignment
     - Return the longest alignment among the alignment satisfying the similarity cutoff is output as a result.

# Quick Start
```rust
```
*/

use anyhow::{Result, bail as error_msg};
use serde::{Serialize, Deserialize, de::DeserializeOwned};

#[cfg(target_endian = "little")]
type EndianType = byteorder::LittleEndian;
#[cfg(target_endian = "big")]
type EndianType = byteorder::BigEndian;
#[cfg(target_pointer_width = "64")]
type SizedUint = u64;
#[cfg(target_pointer_width = "32")]
type SizedUint = u32;

#[doc(hidden)]
// Core
mod core;
// Reference
mod reference;
// Aligner
mod aligner;
// Builder
mod builder;
// Result
pub mod result;
// Attachment
mod attachment;
#[doc(hidden)]
pub mod deprecated;
#[doc(hidden)]
pub mod util;
// #[cfg(test)]
// mod tests;


// Publics
pub use reference::{Reference, sequence_provider};
pub use aligner::Aligner;
pub use builder::ReferenceBuilder;

mod example {
    #[test]
    fn test_get_result_tmp() {
        use crate::{Aligner, ReferenceBuilder};
        use crate::sequence_provider::InMemoryProvider;

        // (1) Define `SequenceProvider`
        let mut sequence_provider = InMemoryProvider::new();
        sequence_provider.add_record(
            b"ATCAAACTCACAATTGTATTTCTTTGCCAGCTGGGCATATACTTTTTCCGCACCCTCATTTAACTTCTTGGATAACGGAAGCACACCGATCTTAACCGGAGCAAGTGCCGGATGAAAATGGAAAACGGTTCTTACGTCCGGCTTTTCCTCTGTTCCGATATTTTCCTCATCGTATGCAGCACATAAAAATGCCAGAACCA",
            "record_1"
        );
        sequence_provider.add_record(
            b"TTCCATCAAACTCACAATTGTATTTCTTTGCCAGCTGGGCATATACTTTTTCCGCACCCTCATTTAACTTCTTGGATAACGGAAGCACACCGATCTTAACCGGAGCGTATGCAGCACATAAAAAT",
            "record_2",
        );

        // (2) Make `Reference`
        let reference = ReferenceBuilder::new().build(sequence_provider).unwrap();

        // (3) Make `Aligner`
        let mut aligner = Aligner::new_local(4, 6, 2, 100, 0.1).unwrap();

        // (4) Alignment
        let query = b"TTCCTCTGTCATCAAACTCACAATTGTATTTCTTTGCCAGCTGGGCATATACTTTTTCCGCCCCCTCATTTAACTTCTTGGATAACGGAAGCACACCGATCTTAACCGGAGGTGCCGGATGAAAATGGAAAACGGTTCTTACGTCCGGCTTTTCCTCTGTTCCGATATTTTCCTCAT";
        let result = aligner.query_alignment(&reference, query).unwrap();
        println!("{}", result.to_json_pretty());
    }
    // #[test]
    // fn print_library_example_quick_start() {
    //     use crate::{Reference, Aligner};
    //     use crate::basic_sequence_provider::InMemoryProvider;

    //     // (1) Make `Reference`
    //     let mut sequence_provider = InMemoryProvider::new_empty();
    //     sequence_provider.add_labeled_sequence(
    //         "record_1".to_string(),
    //         b"ATCAAACTCACAATTGTATTTCTTTGCCAGCTGGGCATATACTTTTTCCGCACCCTCATTTAACTTCTTGGATAACGGAAGCACACCGATCTTAACCGGAGCAAGTGCCGGATGAAAATGGAAAACGGTTCTTACGTCCGGCTTTTCCTCTGTTCCGATATTTTCCTCATCGTATGCAGCACATAAAAATGCCAGAACCA".to_vec(),
    //     );
    //     sequence_provider.add_labeled_sequence(
    //         "record_2".to_string(),
    //         b"TTCCATCAAACTCACAATTGTATTTCTTTGCCAGCTGGGCATATACTTTTTCCGCACCCTCATTTAACTTCTTGGATAACGGAAGCACACCGATCTTAACCGGAGCGTATGCAGCACATAAAAAT".to_vec(),
    //     );
    //     let mut reference = Reference::new_with_default_config(sequence_provider).unwrap();

    //     // (2) Make `Aligner`
    //     let mut aligner = Aligner::new(4, 6, 2, 100, 0.1).unwrap();

    //     // (3) Alignment with query
    //     let query = b"TTCCTCTGTCATCAAACTCACAATTGTATTTCTTTGCCAGCTGGGCATATACTTTTTCCGCCCCCTCATTTAACTTCTTGGATAACGGAAGCACACCGATCTTAACCGGAGGTGCCGGATGAAAATGGAAAACGGTTCTTACGTCCGGCTTTTCCTCTGTTCCGATATTTTCCTCAT";
    //     // - Semi-global alignment
    //     let result_semi_global: String = aligner.semi_global_alignment_labeled(&mut reference, query).unwrap();
    //     // - Local alignment
    //     let result_local: String = aligner.local_alignment_labeled(&mut reference, query).unwrap();
    // }
}