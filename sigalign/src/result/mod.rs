use crate::{Result, error_msg};
use crate::{Serialize, Deserialize};
use crate::core::{
	Penalties, PRECISION_SCALE, Cutoff, MinPenaltyForPattern,
    Sequence,
    ReferenceInterface, SequenceBuffer, PatternLocation,
};
pub use crate::core::{
    AlignmentResult,
    RecordAlignmentResult,
    AnchorAlignmentResult,
    AlignmentPosition,
    AlignmentOperation,
    AlignmentCase,
};
pub(crate) use crate::reference::{
    Reference,
    SequenceProvider,
    LabelProvider,
};

// Result structures
mod fasta_result;
pub use fasta_result::{
    FastaAlignmentResult,
    ReadAlignmentResult,
};
mod labeled_result;
pub use labeled_result::{
    FastaAlignmentLabeledResult,
    ReadAlignmentLabeledResult,
    AlignmentLabeledResult,
    RecordAlignmentLabeledResult,
};

// Features
mod result_counts;

// Encoders
mod to_json;
mod to_labeled;
