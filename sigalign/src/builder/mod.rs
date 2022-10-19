use crate::{Result, error_msg};
use crate::core::{
	Penalties, PRECISION_SCALE, Cutoff, MinPenaltyForPattern,
	AlignmentResult, RecordAlignmentResult, AnchorAlignmentResult, AlignmentPosition, AlignmentOperation, AlignmentCase,
    Sequence,
    ReferenceInterface, SequenceBuffer, PatternLocation,
};
use crate::reference::{
    Reference, SequenceStorage, SequenceType, JoinedSequence, PatternFinder,
};

// Builders
mod reference;
pub use reference::ReferenceBuilder;
mod aligner;
