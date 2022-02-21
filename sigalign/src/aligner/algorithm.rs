// Alignment algorithms
use super::{
	Penalties, PRECISION_SCALE, Cutoff, MinPenaltyForPattern,
	AlignmentResult, RecordAlignmentResult, AnchorAlignmentResult, AlignmentPosition, AlignmentOperation, AlignmentCase,
    Sequence,
    ReferenceInterface, SequenceBuffer, PatternLocation,
    Reference, SequenceProvider,
};

mod common_steps_dep;
use common_steps_dep::{AlignmentHashSet};

mod semi_global_dep;
mod local_dep;

pub use local_dep::local_alignment_algorithm;
pub use semi_global_dep::semi_global_alignment_algorithm;

// New version!
// Common steps
mod pos_table;
use pos_table::{PosTable, AnchorPosition, AnchorIndex};
mod extending;
use extending::{Extension, WaveEndPoint, WaveFrontScore, Components, Component, BackTraceMarker, calculate_spare_penalty_from_determinant};
pub use extending::WaveFront;
mod traversing;
use traversing::{Traversed};


mod semi_global;