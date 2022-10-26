# SigAlign
***Si**milarity-**g**uided **Align***

## Key Features
1. Simplified parameters
    * **3** gap-affine penalties
        1. Mismatch penalty
        2. Gap-open penalty
        3. Gap-extend penalty
    * **2** similarity cut-offs
        1. Minimum length
        2. Maximum penalty per length
2. Non-heuristic
    * No exception for similarity cut-offs.
        * If there is an alignment result, the optimal alignment is always included in the result. <-> If there is no result, the optimal alignment does not satisfy the similarity cut-offs. 
3. Reproducible algorithm
    * *Semi-global*
        1.	Generate dynamic programming (DP) matrix.
        2.	Sort out the most optimal semi-global alignment from DP matrix.
        3.	Repeatedly sort out the next optimal alignment of which location is not overlapped with previous alignments.
        4.	Print all alignments satisfying cut-offs.
    * *Local*
        1. For all substrings of query, perform the semi-global alignment.

## Quick Start
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
