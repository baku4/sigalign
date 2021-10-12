use crate::core::{Penalties, Cutoff, MinPenaltyForPattern};
use crate::core::{Sequence, Reference, PatternLocation};
use crate::core::{AlignmentResultsByRecord, AlignmentResult, AlignmentPosition, AlignmentOperation, AlignmentType};
use crate::core::{Anchors};
use crate::core::{DropoffWaveFront, WaveFrontScore, Components, Component};
use crate::core::{M_COMPONENT, I_COMPONENT, D_COMPONENT, EMPTY, FROM_M, FROM_I, FROM_D, START};

mod semi_global;
mod local;