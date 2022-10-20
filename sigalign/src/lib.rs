/*!
# SigAlign
*Similarity-guided Align*

# Quick Start
```rust
use crate::{Aligner, ReferenceBuilder};
use crate::sequence_storage::InMemoryStorage;

// (1) Build `Reference`
//  - Make `SequenceStorage`
let mut sequence_storage = InMemoryStorage::new();
sequence_storage.add_record(
    b"ACACAGATCGCAAACTCACAATTGTATTTCTTTGCCACCTGGGCATATACTTTTTGCGCCCCCTCATTTA",
    "record_1"
);
sequence_storage.add_record(
    b"TCTGGGGCCATTGTATTTCTTTGCCAGCTGGGGCATATACTTTTTCCGCCCCCTCATTTACGCTCATCAC",
    "record_2",
);
//  - Pass `SequenceStorage` to `ReferenceBuilder`
let reference = ReferenceBuilder::new().build(sequence_storage).unwrap();

// (2) Make `Aligner`
let mut aligner = Aligner::new_local(
    4,   // Mismatch penalty
    6,   // Gap-open penalty
    2,   // Gap-extend penalty
    50,  // Minimum aligned length
    0.2, // Maximum penalty per length
).unwrap();

// (3) Align query to reference
let query = b"CAAACTCACAATTGTATTTCTTTGCCAGCTGGGCATATACTTTTTCCGCCCCCTCATTTAACTTCTTGGA";
let result = aligner.query_alignment(&reference, query).unwrap();
println!("{}", result.to_json());
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
// (1) Core structures
pub mod core;
// (2) Reference implementation
mod reference;
// (3) Aligner and result
mod aligner;
/// Structure of alignment result
pub mod result;
// (4) Public APIs
mod alignment; // Alignment functions
mod builder; // Struct builders
/// Utilities
pub mod util;

/*
Re-export publics
*/
pub use reference::{Reference, sequence_storage};
pub use aligner::Aligner;
pub use builder::ReferenceBuilder;

// For unit tests
mod tests;

mod example {
    use std::io::Cursor;

    #[test]
    fn test_get_result_tmp() {
        use crate::{Aligner, ReferenceBuilder};
        use crate::sequence_storage::InMemoryStorage;

        // (1) Build `Reference`
        //  - Make `SequenceStorage`
        let mut sequence_storage = InMemoryStorage::new();
        sequence_storage.add_record(
            b"ACACAGATCGCAAACTCACAATTGTATTTCTTTGCCACCTGGGCATATACTTTTTGCGCCCCCTCATTTA",
            "record_1"
        );
        sequence_storage.add_record(
            b"TCTGGGGCCATTGTATTTCTTTGCCAGCTGGGGCATATACTTTTTCCGCCCCCTCATTTACGCTCATCAC",
            "record_2",
        );
        //  - Pass `SequenceStorage` to `ReferenceBuilder`
        let reference = ReferenceBuilder::new().build(sequence_storage).unwrap();

        // (2) Make `Aligner`
        let mut aligner = Aligner::new_local(
            4, // (1) Mismatch penalty
            6, // (2) Gap-open penalty
            2, // (3) Gap-extend penalty
            50, // (4) Minimum aligned length
            0.2, // (5) Maximum penalty per length
        ).unwrap();

        // (3) Align query to reference
        let query = b"CAAACTCACAATTGTATTTCTTTGCCAGCTGGGCATATACTTTTTCCGCCCCCTCATTTAACTTCTTGGA";
        let result = aligner.query_alignment(&reference, query).unwrap();
        println!("{}", result.to_json());
    }

    #[test]
    fn test_save_and_load_tmp() {
        use crate::{Aligner, Reference, ReferenceBuilder};
        use crate::sequence_storage::InMemoryStorage;

        let mut sequence_storage = InMemoryStorage::new();
        sequence_storage.add_record(
            b"ACACAGATCGCAAACTCACAATTGTATTTCTTTGCCACCTGGGCATATACTTTTTGCGCCCCCTCATTTA",
            "record_1"
        );
        sequence_storage.add_record(
            b"TCTGGGGCCATTGTATTTCTTTGCCAGCTGGGGCATATACTTTTTCCGCCCCCTCATTTACGCTCATCAC",
            "record_2",
        );
        let reference = ReferenceBuilder::new().build(sequence_storage).unwrap();

        let mut buffer: Vec<u8> = Vec::new();


        reference.save_to(&mut buffer).unwrap();

        println!("{}", buffer.len());
        let cursor = Cursor::new(buffer);

        let loaded_reference: Reference<InMemoryStorage> = Reference::load_from(cursor).unwrap();

        let mut aligner = Aligner::new_local(4, 6, 2, 50, 0.2).unwrap();
        let query = b"CAAACTCACAATTGTATTTCTTTGCCAGCTGGGCATATACTTTTTCCGCCCCCTCATTTAACTTCTTGGA";
        let result = aligner.query_alignment(&reference, query).unwrap();
        println!("{}", result.to_json());
        let result = aligner.query_alignment(&loaded_reference, query).unwrap();
        println!("{}", result.to_json());
    }
}
