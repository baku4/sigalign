use crate::core::ReferenceInterface;
use crate::results::AlignmentResult;
use super::{
    AlignerInterface,
    AlignmentRegulator, RegulatorError,
    WaveFrontPool, SingleWaveFrontPool, AllocationStrategy,
    semi_global_alignment_algorithm,
};

#[derive(Debug, Clone)]
pub struct SemiGlobalAligner<A: AllocationStrategy> {
    regulator: AlignmentRegulator,
    internal_buffer: SingleWaveFrontPool<A>,
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
            internal_buffer: wave_front_pool,
        })
    }
    fn alignment<R: ReferenceInterface>(
        &mut self,
        reference: &R,
        sequence_buffer: &mut R::Buffer,
        query: &[u8],
    ) -> AlignmentResult {
        self.internal_buffer.allocate_if_needed(
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
            &mut self.internal_buffer.wave_front,
        );

        self.regulator.result_of_uncompressed_penalty(reference_alignment_result)
    }

    fn get_mismatch_penalty(&self) -> u32 {
        self.regulator.get_mismatch_penalty()
    }
    fn get_gap_open_penalty(&self) -> u32 {
        self.regulator.get_gap_open_penalty()
    }
    fn get_gap_extend_penalty(&self) -> u32 {
        self.regulator.get_gap_extend_penalty()
    }
    fn get_minimum_aligned_length(&self) -> u32 {
        self.regulator.get_minimum_aligned_length()
    }
    fn get_maximum_penalty_per_length(&self) -> f32 {
        self.regulator.get_maximum_penalty_per_length()
    }
    fn get_pattern_size(&self) -> u32 {
        self.regulator.get_pattern_size()
    }
}
