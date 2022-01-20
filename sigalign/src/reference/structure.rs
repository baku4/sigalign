use super::{
    Result, error_msg,
	Penalties, PRECISION_SCALE, Cutoff, MinPenaltyForPattern,
	ReferenceAlignmentResult, RecordAlignmentResult, AlignmentResult, AlignmentPosition, AlignmentOperation, AlignmentType,
    Sequence,
    ReferenceInterface, PatternLocation,
    AlignerInterface,
};
use std::io::{Write, Read};

mod pattern_finder;
mod sequence_type;

pub use pattern_finder::{PatternFinder, JoinedSequence};
pub use sequence_type::SequenceType;

trait FixedSizeEncoding {
    const BYTE_SIZE_OF_STRUCTURE: usize;
    
    fn write_to<W>(&self, writer: W) -> Result<()> where W: Write;
    fn read_from<R>(reader: R) -> Result<Self> where R: Read, Self: Sized;
}