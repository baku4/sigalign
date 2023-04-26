pub use crate::core::{PatternLocation};
use super::{SequenceType};

pub trait PatternIndex: Sized {
    type Option;

    fn new(
        concatenated_sequence_with_boundaries: ConcatenatedSequenceWithBoundaries,
        sequence_type: &SequenceType,
        option: Self::Option,
    ) -> Result<Self, PatternIndexBuildError>;
    fn locate(&self, pattern: &[u8], search_range: &Vec<u32>) -> Vec<PatternLocation>;
}
/// If there are three targets with "ATT", "CC", "GGGG", the "concatenated_sequence" is "ATTCCGGGG" and the "boundaries" is [0, 3, 5, 9].
pub struct ConcatenatedSequenceWithBoundaries {
    pub concatenated_sequence: Vec<u8>,
    pub boundaries: Vec<u64>,
}

use thiserror::Error;
#[derive(Debug, Error)]
pub enum PatternIndexBuildError {
    #[error("Sequence length is over the maximum capacity {0}")]
    SequenceLengthOver(u64), // Maximum capacity
    #[error("Unsupported sequence types: {0}")]
    UnsupportedSequenceType(String), // Concatenated unsupported sequence
    #[error("Pattern index can make index of {max} characters, input is {input}")]
    OverMaximumCharacters{
        max: u32,    // The maximum number of characters that PatternIndex can index
        input: u32,  // Input characters
    },
    #[error("Error in option: {0}")]
    Option(String), // Error message
}

/// Implementations for [PatternIndex]
pub mod implementations;
