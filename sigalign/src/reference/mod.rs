use crate::{Result, error_msg};
// TODO: Delete unused_imports
#[allow(unused_imports)]
use crate::core::{
	Penalty, PREC_SCALE, Cutoff, MinPenaltyForPattern,
	AlignmentResult, RecordAlignmentResult, AnchorAlignmentResult, AlignmentPosition, AlignmentOperation, AlignmentCase,
    Sequence,
    ReferenceInterface, PatternLocation,
};

// Requirements for inner structures
mod requirements;
pub use requirements::{ // TODO: `pub` can be deleted?
    // Traits
    Serialize,
    EstimateSize,
    Divide,
};
// Common data structures
mod commons;
pub use commons::{
    SequenceType,
    JoinedSequence,
    PatternFinder,
};
// Storage of sequences
pub mod sequence_storage;
pub use sequence_storage::{
    SequenceStorage,
};

// Default features of Reference
mod feature;
pub use feature::{
    // For sequence storage
    LabelStorage,
    RcStorage,
};

// To display the hyperlink in Documentation
#[allow(unused_imports)]
use super::{ReferenceBuilder, Aligner};
/**
The database for multiple targeted sequences.

- `Reference` contains:
    1. Supported type of sequence
    2. Index of sequences
    3. Range to search
    4. [SequenceStorage]

- `Reference` is **immutable** that can be safely shared by multiple threads.

- Basic usage
    1. Built from (1) [ReferenceBuilder] with (2) [SequenceStorage].
    2. Pass to [Aligner] to perform alignment.

- Advanced usage
    - Change range of reference searching
        - The `search range` is a `Vec` of indexes of records to be searched.
            - Right after the reference built, it is set for the entire record (0..the number of records).
    - 
*/
#[derive(Debug)]
pub struct Reference<S> where
    S: SequenceStorage,
{
    pub sequence_type: SequenceType,
    pub pattern_finder: PatternFinder,
    pub target_record_index: Vec<u32>,
    pub sequence_storage: S,
}

impl<S> Reference<S> where
    S: SequenceStorage,
{
    /// CAVEAT.
    /// This is raw implementation for specific scenarios for performance optimization.
    /// The safe method to build Reference is to use [ReferenceBuilder].
    pub fn new(
        sequence_type: SequenceType,
        pattern_finder: PatternFinder,
        target_record_index: Vec<u32>,
        sequence_storage: S
    ) -> Self {
        Self {
            sequence_type,
            pattern_finder,
            target_record_index,
            sequence_storage: sequence_storage,
        }
    }
}
