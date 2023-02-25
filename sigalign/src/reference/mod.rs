/*!
The database for multiple targeted sequences
# Reference
Reference is the database for multiple targeted sequences.
## Features
Reference has two main features:
  1. Get the sequence of the target by index
  2. Locate the exactly matched pattern in all target.
### 1. Getting the sequence
Reference is immutable to be safely shared between multiple threads. To accomplish immutability, getting the sequence is processed outside of the reference by following steps:
  1. Get the buffer of sequence (In most case, Aligner gets the buffer).
  2. Fill the sequence to buffer.
  3. Request the sequence to buffer.
The implementation of how to get, fill and request sequence can differ by use cases. Therefore, the SequenceStorage that deals with the sequences is `trait`, not a `struct`.
### 2. Getting the sequence
Locating the pattern is necessary to algorithm. In algorithm, the sequence is divided into several patterns and each patterns locate in the all target. Locating is getting the index of target sequence which is exactly matched with the pattern. PatternIndex takes this job using FmIndex.
## Fields
Reference has four fields:
  1. SequenceType
  2. PatternIndex
  3. SearchRange
  4. SequenceStorage
### 1. SequenceType
SequenceType is the definition for the valid sequences. To compress the index and accelerate the speed of locating pattern, Reference makes index only with the input characters set. SequenceType remembers the input characters, and can check whether the sequence is indexed or not.
### 2. PatternIndex
PatternIndex can locate the exactly matched pattern in all sequence. This is defined as trait. In early version of SigAlign, PatternIndex is solid struct, but some issues are here:
  1. The performance of PatternIndex takes the large amount of the overall performance, especially for the short query.
  2. Inner structure of PatternIndex is frequently changed. The LtFmIndex, Rust crate for pattern matching, is used in SigAlign and this crate is actively developing.
### 3. SearchRange
SearchRange is the **sorted** index of targets. This can be modified after building the reference. Adjusting the search range is useful when large reference is built and used in case that only needs the subset of reference without re-building the subset of reference.
### 4. SequenceStorage
SequenceStorage is the storage of all targets sequences. SequenceStorage is defined as trait. The implementation detail of storing and parsing the sequence can be optimize for various scenarios. SigAlign has the default implementations of SequenceStorage. InMemoryStorage is one of them to store all sequence into the memory.
*/
use crate::core::{ReferenceInterface, PatternLocation};

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

pub mod sequence_type;
use sequence_type::SequenceType;
pub mod pattern_index;
use pattern_index::{PatternIndex, ConcatenatedSequenceWithBoundaries, PatternIndexBuildError};
pub mod sequence_storage;
use sequence_storage::SequenceStorage;

impl<I, S> ReferenceInterface for Reference<I, S> where
    I: PatternIndex,
    S: SequenceStorage,
{
    type Buffer = S::Buffer;

    fn locate(&self, pattern: &[u8]) -> Vec<PatternLocation> {
        self.pattern_index.locate(pattern, &self.search_range)
    }
    fn get_buffer(&self) -> Self::Buffer {
        self.sequence_storage.get_buffer()
    }
    fn fill_buffer(&self, target_index: u32, buffer: &mut Self::Buffer) {
        self.sequence_storage.fill_buffer(target_index, buffer)
    }
    fn is_valid(&self, query: &[u8]) -> bool {
        self.sequence_type.validate_query(query)
    }
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
        let sequence_type = SequenceType::new(&concatenated_sequence_with_boundaries.concatenated_sequence);
        let pattern_index = I::new(concatenated_sequence_with_boundaries, &sequence_type, pattern_index_option)?;
        let num_targets = sequence_storage.num_targets();
        let search_range: Vec<u32> = (0..num_targets).collect();

        Ok(Self {
            sequence_type,
            search_range,
            pattern_index,
            sequence_storage,
        })
    }
    pub fn from_raw_unchecked(
        sequence_type: SequenceType,
        search_range: Vec<u32>,
        pattern_index: I,
        sequence_storage: S,
    ) -> Self {
        Self {
            sequence_type,
            search_range,
            pattern_index,
            sequence_storage,
        }
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

pub mod features;

// // Requirements for inner structures
// mod requirements;
// use requirements::{
//     Serialize,
//     EstimateSize,
// };
// // Common data structures
// mod commons;
// pub use commons::{
//     SequenceType,
//     JoinedSequence,
//     PatternFinder,
// };
// // Storage of sequences
// pub mod sequence_storage;
// pub use sequence_storage::{
//     SequenceStorage,
// };

// // Default features of Reference
// mod feature;
// pub use feature::{
//     // For sequence storage
//     LabelStorage,
//     RcStorage,
// };
