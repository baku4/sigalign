use super::{
    Result, error_msg,
	Penalties, PRECISION_SCALE, Cutoff, MinPenaltyForPattern,
	AlignmentResult, RecordAlignmentResult, AnchorAlignmentResult, AlignmentPosition, AlignmentOperation, AlignmentCase,
    Sequence,
    ReferenceInterface, SequenceBuffer, PatternLocation,
};
use super::{
    Reference, SequenceStorage,
    // Requirement for struct
    Serializable, SizeAware,
    // Basic struct
    SequenceType, PatternFinder,
};

mod new;
mod reference_interface;
mod set_search_range;
mod io;
mod labeling;
mod reverse_complement;
mod debug;

// For sequence storage
pub use labeling::LabelStorage;
pub use reverse_complement::ReverseComplement;
