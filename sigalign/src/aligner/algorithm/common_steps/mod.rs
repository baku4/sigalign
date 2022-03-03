use super::{
	Penalties, PRECISION_SCALE, Cutoff, MinPenaltyForPattern,
	AlignmentResult, RecordAlignmentResult, AnchorAlignmentResult, AlignmentPosition, AlignmentOperation, AlignmentCase,
    Sequence,
    ReferenceInterface, SequenceBuffer, PatternLocation,
    Reference, SequenceProvider,
};

mod pos_table;
mod spare_penalty;
mod extend;
mod backtrace;
mod merging;

pub use pos_table::{PosTable, AnchorPosition, AnchorIndex};
pub use spare_penalty::{calculate_spare_penalty};
pub use extend::{Extension, WaveFront, WaveEndPoint, WaveFrontScore, Components, Component, BackTraceMarker};
pub use backtrace::{TraversedPosition, TraversedAnchors, TraversedAnchor};
