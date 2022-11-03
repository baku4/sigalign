// Alignment algorithms
use super::{
	Penalties, PRECISION_SCALE, Cutoff, MinPenaltyForPattern,
	AlignmentResult, RecordAlignmentResult, AnchorAlignmentResult, AlignmentPosition, AlignmentOperation, AlignmentCase,
    Sequence,
    ReferenceInterface, SequenceBuffer,
    Reference, SequenceStorage,
};

mod common_steps;
pub use common_steps::WaveFront;
// TODO: Delete unused_imports
#[allow(unused_imports)]
use common_steps::{
    PosTable, AnchorPosition, AnchorIndex,
    calculate_spare_penalty,
    Extension, WaveEndPoint, WaveFrontScore, Components, Component, BackTraceMarker,
    TraversedPosition, TraversedAnchor,
};

mod local;
pub use local::local_alignment_algorithm;
mod semi_global;
pub use semi_global::semi_global_alignment_algorithm;
