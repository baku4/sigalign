use super::{
    Result, error_msg,
	Penalties, PRECISION_SCALE, Cutoff, MinPenaltyForPattern,
	AlignmentResult, RecordAlignmentResult, AnchorAlignmentResult, AlignmentPosition, AlignmentOperation, AlignmentCase,
    Sequence,
    ReferenceInterface, SequenceBuffer, PatternLocation,
};
use super::{
    Reference, SequenceProvider,
    SequenceType, PatternFinder,
};

use std::io::{Write, Read};

pub trait Serializable {
    fn save_to<W>(&self, writer: W) -> Result<()> where W: Write;
    fn load_from<R>(reader: R) -> Result<Self> where R: Read, Self: Sized;
}

// Precalculate stored size
pub trait SizeAware {
    fn size_of(&self) -> usize;
}
