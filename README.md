# SigAlign
***Si**milarity-**g**uided **Align***

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
