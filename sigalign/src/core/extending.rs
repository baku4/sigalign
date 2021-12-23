use super::Penalties;
use super::Sequence;
use super::{AlignmentOperation, AlignmentType};

mod dwfa;

mod wave_front;

pub use wave_front::{WaveFront, EndPoint, WaveFrontScore, Components, Component, BackTraceMarker};

#[derive(Debug, Clone)]
pub struct Extension {
    pub penalty: usize,
    pub length: usize,
    pub insertion_count: u32,
    pub deletion_count: u32,
    pub operations: Vec<AlignmentOperation>,
}
