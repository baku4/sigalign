use super::Penalties;
use super::Sequence;
use super::{AlignmentOperation, AlignmentType};

mod dwfa;

// pub use dwfa::{DropoffWaveFront, WaveFrontScore, Components, Component};
// pub use dwfa::{M_COMPONENT, I_COMPONENT, D_COMPONENT, EMPTY, FROM_M, FROM_I, FROM_D, START};

mod wave_front;

pub use wave_front::{WaveFront, EndPoint, WaveFrontScore, Components, Component, MatchBt, InsBt, DelBt};

#[derive(Debug, Clone)]
pub struct Extension {
    pub penalty: usize,
    pub length: usize,
    pub insertion_count: u32,
    pub deletion_count: u32,
    pub operations: Vec<AlignmentOperation>,
}
