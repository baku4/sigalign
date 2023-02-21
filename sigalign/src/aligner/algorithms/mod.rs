use crate::core::{
    regulators::{
        Penalty, PREC_SCALE, Cutoff, MinPenaltyForPattern,
    },
    results::{
        AlignmentResult, TargetAlignmentResult, AnchorAlignmentResult, AlignmentPosition, AlignmentOperations, AlignmentOperation,
    },
    ReferenceInterface, SequenceBuffer, PatternLocation,
};

mod common_steps;
pub use common_steps::WaveFront;

mod local;
pub use local::local_alignment_algorithm;
mod semi_global;
pub use semi_global::semi_global_alignment_algorithm;
