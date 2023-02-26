use crate::core::ReferenceInterface;
use crate::results::AlignmentResult;
use super::{
    AlignerInterface,
    AlignmentRegulator, RegulatorError,
    WaveFrontPool, SingleWaveFrontPool, AllocationStrategy,
    semi_global_alignment_algorithm,
};

#[derive(Clone)]
pub struct SemiGlobalAligner<A: AllocationStrategy> {
    pub regulator: AlignmentRegulator,
    pub wave_front_pool: SingleWaveFrontPool<A>,
}

impl<A: AllocationStrategy> AlignerInterface for SemiGlobalAligner<A> {
    fn new(
        mismatch_penalty: u32,
        gap_open_penalty: u32,
        gap_extend_penalty: u32,
        minimum_aligned_length: u32,
        maximum_penalty_per_length: f32,
    ) -> Result<Self, RegulatorError> {
        let regulator = AlignmentRegulator::new(mismatch_penalty, gap_open_penalty, gap_extend_penalty, minimum_aligned_length, maximum_penalty_per_length)?;
        let wave_front_pool = SingleWaveFrontPool::new(&regulator.penalties, &regulator.cutoff);
        Ok(Self {
            regulator,
            wave_front_pool,
        })
    }
    fn alignment<R: ReferenceInterface>(
        &mut self,
        reference: &R,
        sequence_buffer: &mut R::Buffer,
        query: &[u8],
    ) -> AlignmentResult {
        self.wave_front_pool.allocate_if_needed(
            query.len() as u32,
            &self.regulator.penalties,
            &self.regulator.cutoff,
        );
        let reference_alignment_result = semi_global_alignment_algorithm(
            reference,
            sequence_buffer,
            query,
            self.regulator.pattern_size,
            &self.regulator.penalties,
            &self.regulator.min_penalty_for_pattern,
            &self.regulator.cutoff,
            &mut self.wave_front_pool.wave_front,
        );

        self.regulator.result_of_uncompressed_penalty(reference_alignment_result)
    }
}
