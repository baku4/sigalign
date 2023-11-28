use crate::results::AlignmentResult;
use crate::reference::{
    Reference, PatternIndex, SequenceStorage,
};

// Internal structures for Aligner
mod regulator;
pub use regulator::{AlignmentRegulator, RegulatorError};

mod space_manager;
use space_manager::{
    // Single
    SingleSpaceManager,
    SingleLocalSpaceManager,
    SingleSemiGlobalSpaceManager,
    // Multiple
    MultipleSpaceManager,
    MultipleLocalSpaceManager,
    MultipleSemiGlobalSpaceManager,
};
pub use space_manager::AllocationStrategy;

// Aligner
mod local;
pub use local::LocalAligner;
mod local_with_limit;
pub use local_with_limit::LocalWithLimitAligner;
mod local_chaining;
pub use local_chaining::LocalChainingAligner;
mod semi_global;
pub use semi_global::SemiGlobalAligner;
mod semi_global_with_limit;
pub use semi_global_with_limit::SemiGlobalWithLimitAligner;
mod semi_global_chaining;
pub use semi_global_chaining::SemiGlobalChainingAligner;

mod switch_modes;

pub trait Aligner {
    /// Low-level alignment function
    fn alignment<I: PatternIndex, S: SequenceStorage> (
        &mut self,
        reference: &Reference<I, S>,
        sequence_buffer: &mut S::Buffer,
        sorted_target_indices: &[u32],
        query: &[u8],
    ) -> AlignmentResult;
}
