use crate::core::ReferenceInterface;
use crate::results::AlignmentResult;
use super::{
    AlignerInterface,
    AlignmentRegulator, RegulatorError,
    WaveFrontPool, SingleWaveFrontPool,
    AllocationStrategy, QueryLengthChecker,
    semi_global_alignment_algorithm,
};

#[derive(Debug, Clone)]
pub struct SemiGlobalAligner<A: AllocationStrategy> {
    query_length_checker: QueryLengthChecker<A>,
    regulator: AlignmentRegulator,
    wave_front_pool: SingleWaveFrontPool,
}

impl<A: AllocationStrategy> AlignerInterface for SemiGlobalAligner<A> {
    fn new(
        mismatch_penalty: u32,
        gap_open_penalty: u32,
        gap_extend_penalty: u32,
        minimum_aligned_length: u32,
        maximum_penalty_per_length: f32,
    ) -> Result<Self, RegulatorError> {
        let query_length_checker = QueryLengthChecker::new();
        let regulator = AlignmentRegulator::new(mismatch_penalty, gap_open_penalty, gap_extend_penalty, minimum_aligned_length, maximum_penalty_per_length)?;
        // Create buffers
        let query_length = query_length_checker.get_allocated_length();
        let wave_front_pool = SingleWaveFrontPool::new(
            query_length,
            &regulator.penalties,
            &regulator.cutoff,
        );
        Ok(Self {
            query_length_checker,
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
        if let Some(v) = self.query_length_checker.optional_length_to_be_allocated(query.len() as u32) {
            self.wave_front_pool.allocate(
                v,
                &self.regulator.penalties,
                &self.regulator.cutoff,
            );
        }
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
