use super::{
	Penalty, PREC_SCALE, Cutoff,
    AlignmentOperation, AlignmentCase,
    Sequence,
    ReferenceInterface,
    Reference, SequenceStorage,
};

mod pos_table;
mod spare_penalty;
mod extend;
mod backtrace;
mod merging;

pub use pos_table::{PosTable, AnchorPosition, AnchorIndex};
pub use spare_penalty::{calculate_spare_penalty};
pub use extend::{Extension, WaveFront, WaveEndPoint, WaveFrontScore, Components, Component, BackTraceMarker};
pub use backtrace::{TraversedPosition, TraversedAnchor};
