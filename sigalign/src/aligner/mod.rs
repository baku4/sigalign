/*!
Provides the `Aligner` struct, an alignment worker that performs sequence alignment.

## Features

- The `Aligner` conducts sequence alignment, given a query sequence and a `Reference`.
- The creation of an `Aligner` is regulated by alignment parameters: (1) penalties and (2) similarity cutoffs.
- The `Aligner` maintains a reusable workspace, allocating and managing the necessary space for alignment based on the alignment parameters and the length of the query.

## Architecture

The `Aligner` is characterized by two traits: `Mode` and `AllocationStrategy`.

### trait `Mode`

The alignment mode in bioinformatics dictates how sequences are compared and aligned. SigAlign supports two modes: semi-global and local.

In the semi-global mode, either the query or the reference sequence is completely consumed at each alignment end.

For instance:

- Case 1
```text
QUERY : -------------
            |||||||||
TARGET:     -------------
```
- Case 2
```text
QUERY :     -------------
            |||||||||
TARGET: -------------
```
- Case 3
```text
QUERY : -------------
          |||||||
TARGET:   -------
```
- Case 4
```text
QUERY :   -------
          |||||||
TARGET: -------------
```

In the local mode, the alignment may include only parts of the target and query sequence.

For example: 
```text
QUERY : ----------------
               |||||||
TARGET:    ----------------
```

The `Mode` trait is integral to SigAlign's core algorithm, and it is pre-defined with these two modes, not intended for user customization.

### trait `AllocationStrategy`

`AllocationStrategy` determines the strategy for allocating memory for alignment. The memory requirement correlates with the maximum length of the query. For example, if space for aligning a 200-length query is allocated, aligning a 400-length query would require additional allocation. However, if space for a 400-length query is already allocated, it can cover a 200-length query.

`AllocationStrategy` defines the functions necessary for calculating the required space, providing a cap to each strategy as these functions can negatively impact performance if frequently called. For example, LinearStrategy allocates input query length plus a fixed size space, whereas DoublingStrategy allocates approximately twice the needed space. Users can define the `AllocationStrategy` that best suits their requirements.

## Usage

### (1) Initialize `Aligner`
```rust
let mut aligner = Aligner::<LocalMode, LinearStrategy>::new(
    4,   // mismatch_penalty,
    6,   // gap_open_penalty,
    2,   // gap_extend_penalty,
    50,  // minimum_aligned_length,
    0.1, // maximum_penalty_per_length,
).unwrap();
```

### (2) Perform Alignment
#### Using `SequenceBuffer` (Lowest level)
```rust
let mut sequence_buffer = reference.get_sequence_buffer();
for query in [b"AA...CC", b"GG...TT"] {
    aligner.alignment(
        &reference,
        &mut sequence_buffer,
        query,
    );
}
```
#### Using built-in methods
```rust
let result: AlignmentResult = aligner.align_query(
    &reference,
    b"AA..TT",
);
let result: FastaAlignmentResult = aligner.align_fasta_file(
    &reference,
    "FASTA_FILE_PATH",
).unwrap();
```
*/
use crate::results::AlignmentResult;
use crate::reference::{
    Reference,
    pattern_index::PatternIndex,
    sequence_storage::SequenceStorage,
};

// Internal structures for Aligner
mod regulator;
pub(crate) use regulator::AlignmentRegulator;
pub use regulator::RegulatorError;
pub mod allocation_strategy;
use allocation_strategy::{
    QueryLengthChecker,
    AllocationStrategy,
};
pub mod mode;
use mode::AlignmentMode;

pub struct Aligner<M, A> where
    M: AlignmentMode,
    A: AllocationStrategy,
{
    pub(crate) regulator: AlignmentRegulator,
    pub(crate) query_length_checker: QueryLengthChecker<A>,
    pub(crate) mode: M,
}

impl<M, A> Aligner<M, A> where
    M: AlignmentMode,
    A: AllocationStrategy,
{
    pub fn new(
        mismatch_penalty: u32,
        gap_open_penalty: u32,
        gap_extend_penalty: u32,
        minimum_aligned_length: u32,
        maximum_penalty_per_length: f32,
    ) -> Result<Self, RegulatorError> {
        let regulator = AlignmentRegulator::new(mismatch_penalty, gap_open_penalty, gap_extend_penalty, minimum_aligned_length, maximum_penalty_per_length)?;
        let query_length_checker = QueryLengthChecker::new();
        let query_length = query_length_checker.get_allocated_length();

        let mode = M::new(
            query_length,
            &regulator,
        );

        Ok(Self {
            regulator,
            query_length_checker,
            mode,
        })
    }
    pub fn alignment<I: PatternIndex, S: SequenceStorage> (
        &mut self,
        reference: &Reference<I, S>,
        sequence_buffer: &mut S::Buffer,
        query: &[u8],
    ) -> AlignmentResult {
        if let Some(required_query_length) = self.query_length_checker.optional_length_to_be_allocated(query.len() as u32) {
            self.mode.allocate_space(
                required_query_length,
                &self.regulator,
            );
        }
        
        let reference_alignment_result = self.mode.run_algorithm(
            reference,
            sequence_buffer,
            query,
            &self.regulator,
        );

        self.regulator.result_of_uncompressed_penalty(reference_alignment_result)
    }
}

mod debug;
mod alignments;
pub use alignments::AlignmentError;