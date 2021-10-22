use crate::core::{PatternLocation};
use crate::core::{AlignmentHashSet};
use crate::core::{DropoffWaveFront, WaveFrontScore, Components, Component};
use crate::core::{M_COMPONENT, I_COMPONENT, D_COMPONENT, EMPTY, FROM_M, FROM_I, FROM_D, START};

pub use crate::core::{Penalties, Cutoff, MinPenaltyForPattern};
pub use crate::core::{ReferenceInterface, Sequence};
pub use crate::core::{AlignmentResultsByRecord, AlignmentResult, AlignmentPosition, AlignmentOperation, AlignmentType};
pub use crate::core::Algorithm;

mod semi_global;
mod local;

pub use semi_global::SemiGlobalAlgorithm;
pub use local::LocalAlgorithm;
