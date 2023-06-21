/*!
Provides the `PatternIndex` and its basic implementations.
*/

pub use crate::core::PatternLocation;

pub trait PatternIndex: Sized {
    type Option;

    fn new(
        concatenated_sequence_with_boundaries: ConcatenatedSequenceWithBoundaries,
        option: Self::Option,
    ) -> Result<Self, PatternIndexBuildError>;
    fn locate(&self, pattern: &[u8], search_range: &Vec<u32>) -> Vec<PatternLocation>;
}
/**
`ConcatenatedSequenceWithBoundaries` struct provides an efficient representation for multiple sequences
that are concatenated into one for the purpose of indexing, while retaining their original boundaries.

The primary use case for this struct is to act as an interface between `SequenceStorage` and `PatternIndex`, facilitating the generation of a new `PatternIndex`.

While the creation of a `ConcatenatedSequenceWithBoundaries` instance is automatically handled by `SequenceStorage`, it can also be manually implemented for better performance and finer control.

- Fields
  - `concatenated_sequence`: The combined sequence of all target sequences.
  - `boundaries`: Indices indicating the start of each original sequence in the concatenated sequence.
- Example
  - Given three sequences "ATT", "CC", and "GGGG", the `concatenated_sequence` becomes 
"ATTCCGGGG" and the `boundaries` are represented as [0, 3, 5, 9], marking the start of each original sequence.
*/
pub struct ConcatenatedSequenceWithBoundaries {
    pub concatenated_sequence: Vec<u8>,
    pub boundaries: Vec<u64>,
}

use thiserror::Error;
/// Enumerates possible errors encountered when constructing a `PatternIndex`.
#[derive(Debug, Error)]
pub enum PatternIndexBuildError {
    /// Triggered when sequence length exceeds the maximum allowable capacity.
    #[error("Sequence length is over the maximum capacity {0}")]
    SequenceLengthOver(u64), // Maximum capacity

    /// Triggered when unsupported sequence types are encountered.
    #[error("Unsupported sequence types: {0}")]
    UnsupportedSequenceType(String), // Concatenated unsupported sequence

    /// Triggered when input characters exceed the maximum limit that the `PatternIndex` can index.
    #[error("Pattern index can make index of {max} characters, input is {input}")]
    OverMaximumCharacters{
        max: u32,    // The maximum number of characters that PatternIndex can index
        input: u32,  // Input characters
    },

    /// Triggered when the invalid option is passed.
    #[error("Error in option: {0}")]
    InvalidOption(String), // Error message
}

// Implementations for [PatternIndex]
pub mod lfi;
mod utils;
