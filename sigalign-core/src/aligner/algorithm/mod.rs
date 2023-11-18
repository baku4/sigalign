use crate::results::AlignmentResult;
use super::{
    AlignmentRegulator,
    Reference, PatternIndex, SequenceStorage,
};
mod wave_front_pool;
use wave_front_pool::{
    WaveFrontPool,
    SingleWaveFrontPool,
    DoubleWaveFrontPool,
};

mod local;
mod semi_global;
pub use local::Local;
pub use semi_global::SemiGlobal;

pub trait Algorithm: Clone {
    fn new(
        initial_query_length: u32,
        regulator: &AlignmentRegulator,
    ) -> Self;
    fn allocate_space(
        &mut self,
        required_query_length: u32,
        regulator: &AlignmentRegulator,
    );
    fn reset_buffers(
        &mut self,
    );
    fn run_algorithm<I: PatternIndex, S: SequenceStorage>(
        &mut self,
        reference: &Reference<I, S>,
        sequence_buffer: &mut S::Buffer,
        query: &[u8],
        sorted_target_indices: &[u32],
        regulator: &AlignmentRegulator,
    ) -> AlignmentResult;
}
