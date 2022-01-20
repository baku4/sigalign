use super::{
    Result, error_msg,
	Penalties, PRECISION_SCALE, Cutoff, MinPenaltyForPattern,
	ReferenceAlignmentResult, RecordAlignmentResult, AlignmentResult, AlignmentPosition, AlignmentOperation, AlignmentType,
    Sequence,
    ReferenceInterface, PatternLocation,
    AlignerInterface,
};
use super::{
    Reference, SequenceProvider,
    SequenceType, PatternFinder,
};

use std::io::{Write, Read};

impl<S> Reference<S> where
    S: SequenceProvider + Serializable,
{
    fn save_to<W>(&self, writer: W) where W: Write {
        
    }
}

pub trait Serializable {
    fn save_to<W>(&self, writer: W) -> Result<()> where W: Write;
    fn read_from<R>(reader: R) -> Result<Self> where R: Read, Self: Sized;
}