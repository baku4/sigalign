use crate::core::{Penalties, Cutoff, MinPenaltyForPattern};
use crate::core::{Sequence, Reference, PatternLocation};
use crate::core::{AlignmentResultsByRecord, AlignmentResult, AlignmentPosition, AlignmentOperation, AlignmentType};
use crate::core::{Anchors};

mod semi_global;
mod local;