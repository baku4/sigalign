use super::{
    Result, error_msg,
    Sequence,
    ReferenceInterface, PatternLocation,
};
use super::{
    Reference, SequenceStorage,
    // Requirement for struct
    Serialize, EstimateSize,
    // Basic struct
    SequenceType, PatternFinder,
};

mod reference_interface;
mod set_search_range;
mod io;
mod labeling;
mod reverse_complement;
mod debug;

// For sequence storage
pub use labeling::LabelStorage;
pub use reverse_complement::RcStorage;
