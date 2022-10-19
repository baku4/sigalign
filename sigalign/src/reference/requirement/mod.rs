use super::{
    Result, error_msg,
	Penalties, PRECISION_SCALE, Cutoff, MinPenaltyForPattern,
	AlignmentResult, RecordAlignmentResult, AnchorAlignmentResult, AlignmentPosition, AlignmentOperation, AlignmentCase,
    Sequence,
    ReferenceInterface, SequenceBuffer, PatternLocation,
};
use super::{
    Reference, SequenceStorage,
    SequenceType, PatternFinder,
};

mod io;
pub use io::{
    Serializable,
    SizeAware,
};

mod divide;
pub use divide::{
    Divisible
};