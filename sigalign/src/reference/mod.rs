/*!
A database for multiple targeted sequences.
# Reference
## Features

`Reference` has two main features:
  1. Retrieve the target sequence by index.
  2. Locate the exact matching pattern in all target sequences.

### 1. Retrieving the sequence

`Reference` is immutable to be safely shared between multiple threads. To accomplish immutability, retrieving the sequence is processed outside of the `Reference` through the following steps:
  1. Get the sequence buffer (typically done by the `Aligner`).
  2. Fill the sequence into the buffer.
  3. Request the sequence from the buffer.

The implementation of getting, filling, and requesting sequences can vary depending on use cases. Therefore, the `SequenceStorage` that handles the sequences is a `trait` instead of a `struct`.

### 2. Locating the pattern

Pattern locating is necessary for the algorithm. In this algorithm, the sequence is divided into several patterns, and each pattern is located in all target sequences. Locating is the process of getting the index of the target sequence, which exactly matches the pattern. `PatternIndex` is responsible for this task using `FmIndex`.

## Fields

`Reference` has four fields:
  1. `SequenceType`
  2. `PatternIndex`
  3. `SearchRange`
  4. `SequenceStorage`

### 1. SequenceType

`SequenceType` defines valid sequences. To compress the index and accelerate pattern locating, `Reference` creates an index only with the input character set. `SequenceType` stores the input characters and checks whether a sequence is indexed or not.

### 2. PatternIndex

`PatternIndex` locates the exact matching pattern in all sequences. This is defined as a trait. In early versions of SigAlign, `PatternIndex` was a solid struct, but some issues arose:
  1. The performance of `PatternIndex` accounted for a large portion of the overall performance, especially for short queries.
  2. The inner structure of `PatternIndex` changed frequently. `LtFmIndex`, a Rust crate for pattern matching used in SigAlign, is actively being developed.

### 3. SearchRange

`SearchRange` is the **sorted** index of target sequences. This can be modified after building the reference. Adjusting the search range is useful when a large reference is built and used in cases that only need a subset of the reference without rebuilding the reference subset.

### 4. SequenceStorage

`SequenceStorage` is the storage for all target sequences. `SequenceStorage` is defined as a trait. The implementation details of storing and parsing sequences can be optimized for various scenarios. SigAlign has default implementations of `SequenceStorage`. `InMemoryStorage` is one of them, storing all sequences in memory.
*/
mod sequence_type;
use sequence_type::SequenceType;
pub mod pattern_index;
use pattern_index::{
    PatternIndex,
    ConcatenatedSequenceWithBoundaries,
    PatternIndexBuildError,
};
pub mod sequence_storage;
use sequence_storage::SequenceStorage;

#[derive(Debug)]
pub struct Reference<I, S> where
    I: PatternIndex,
    S: SequenceStorage,
{
    sequence_type: SequenceType,
    search_range: Vec<u32>,
    pattern_index: I,
    sequence_storage: S,
}

impl<I, S> Reference<I, S> where
    I: PatternIndex,
    S: SequenceStorage,
{
    pub fn new(
        sequence_storage: S,
        pattern_index_option: I::Option,
    ) -> Result<Self, ReferenceBuildError> {
        let concatenated_sequence_with_boundaries = sequence_storage.get_concatenated_sequence_with_boundaries();
        let sequence_type = SequenceType::infer_from_sequence(
            &concatenated_sequence_with_boundaries.concatenated_sequence
        );
        let alignable_sequence = sequence_type.alignable_sequence();
        let pattern_index = I::new(
            &alignable_sequence,
            concatenated_sequence_with_boundaries,
            pattern_index_option,
        )?;
        let num_targets = sequence_storage.num_targets();
        let search_range: Vec<u32> = (0..num_targets).collect();

        Ok(Self {
            sequence_type,
            search_range,
            pattern_index,
            sequence_storage,
        })
    }
}

use thiserror::Error;
#[derive(Debug, Error)]
pub enum ReferenceBuildError {
    #[error(transparent)]
    PatternIndexBuildError(#[from] PatternIndexBuildError),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
}

// Features
mod pattern_search;
mod debug;
mod set_search_range;
pub use set_search_range::SetSearchRangeError;
/// Extensions for [Reference]
pub mod extensions;
