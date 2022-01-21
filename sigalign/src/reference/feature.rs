use super::{
    Result, error_msg,
	Penalties, PRECISION_SCALE, Cutoff, MinPenaltyForPattern,
	AlignmentResult, RecordAlignmentResult, AnchorAlignmentResult, AlignmentPosition, AlignmentOperation, AlignmentCase,
    Sequence,
    ReferenceInterface, PatternLocation,
    AlignerInterface,
};
use super::{
    Reference, SequenceProvider,
    SequenceType, PatternFinder, SizeAwareEncoding,
};

mod new;
mod reference_interface;
mod set_search_range;
mod io;

pub use io::Serializable;
