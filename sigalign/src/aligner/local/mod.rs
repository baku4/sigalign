use crate::core::ReferenceInterface;
use crate::results::AlignmentResult;
use super::{
    AlignerInterface,
    AlignmentRegulator, RegulatorError,
    WaveFrontPool, DoubleWaveFrontPool, LinearStrategy, AllocationStrategy,
    local_alignment_algorithm,
};

#[derive(Clone)]
pub struct LocalAligner<A: AllocationStrategy> {
    pub regulator: AlignmentRegulator,
    pub wave_front_pool: DoubleWaveFrontPool<A>,
}

impl<A: AllocationStrategy> AlignerInterface for LocalAligner<A> {
    fn new(
        mismatch_penalty: u32,
        gap_open_penalty: u32,
        gap_extend_penalty: u32,
        minimum_aligned_length: u32,
        maximum_penalty_per_length: f32,
    ) -> Result<Self, RegulatorError> {
        let regulator = AlignmentRegulator::new(mismatch_penalty, gap_open_penalty, gap_extend_penalty, minimum_aligned_length, maximum_penalty_per_length)?;
        let wave_front_pool = DoubleWaveFrontPool::new(&regulator.penalties, &regulator.cutoff);
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
        let reference_alignment_result = local_alignment_algorithm(
            reference,
            sequence_buffer,
            query,
            self.regulator.pattern_size,
            &self.regulator.penalties,
            &self.regulator.cutoff,
            &mut self.wave_front_pool.wave_front_1,
            &mut self.wave_front_pool.wave_front_2,
        );

        self.regulator.result_of_uncompressed_penalty(reference_alignment_result)
    }
}
