use crate::core::{PatternLocation};
use crate::core::{AlignmentHashSet};
use crate::core::{Extension, WaveFront, EndPoint, WaveFrontScore, Components, Component, BackTraceMarker, calculate_spare_penalty_from_determinant};

pub use crate::core::{Penalties, PRECISION_SCALE, Cutoff, MinPenaltyForPattern};
pub use crate::core::{ReferenceInterface, Sequence};
pub use crate::core::{AlignmentResultsByRecordIndex, AlignmentResult, AlignmentPosition, AlignmentOperation, AlignmentType};
pub use crate::core::Algorithm;

mod semi_global;
mod local;

pub use semi_global::SemiGlobalAlgorithm;
pub use local::LocalAlgorithm;
