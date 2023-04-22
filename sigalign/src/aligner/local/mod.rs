use crate::core::ReferenceInterface;
use crate::results::{
    AlignmentResult,
    AlignmentOperations,
};
use super::{
    AlignerInterface,
    AlignmentRegulator, RegulatorError,
    WaveFrontPool, DoubleWaveFrontPool, AllocationStrategy, QueryLengthChecker,
    AnchorIndex, LocalSparePenaltyCalculator,  LocalExtension, Vpc,
    local_alignment_algorithm,
};

#[derive(Debug, Clone)]
pub struct LocalAligner<A: AllocationStrategy> {
    regulator: AlignmentRegulator,
    spare_penalty_calculator: LocalSparePenaltyCalculator,
    query_length_checker: QueryLengthChecker<A>,
    // Buffers
    wave_front_pool: DoubleWaveFrontPool,
    left_vpc_buffer: Vec<Vpc>,
    right_vpc_buffer: Vec<Vpc>,
    sorted_anchor_indices: Vec<AnchorIndex>,
    traversed_anchor_index_buffer: Vec<AnchorIndex>,
    operations_buffer: Vec<AlignmentOperations>,
    extension_buffer: Vec<LocalExtension>,
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
        let query_length_checker = QueryLengthChecker::new();
        let query_length = query_length_checker.get_allocated_length();
        let spare_penalty_calculator = LocalSparePenaltyCalculator::new(
            &regulator.penalties,
            regulator.cutoff.maximum_scaled_penalty_per_length,
            regulator.pattern_size,
            query_length / regulator.pattern_size,
        );
        let wave_front_pool = DoubleWaveFrontPool::new(
            query_length,
            &regulator.penalties,
            &regulator.cutoff,
        );

        Ok(Self {
            regulator,
            query_length_checker,
            spare_penalty_calculator,
            wave_front_pool,
            left_vpc_buffer: Vec::new(),
            right_vpc_buffer: Vec::new(),
            sorted_anchor_indices: Vec::new(),
            traversed_anchor_index_buffer: Vec::new(),
            operations_buffer: Vec::new(),
            extension_buffer: Vec::new(),
            
        })
    }
    fn alignment<R: ReferenceInterface>(
        &mut self,
        reference: &R,
        sequence_buffer: &mut R::Buffer,
        query: &[u8],
    ) -> AlignmentResult {
        if let Some(v) = self.query_length_checker.optional_length_to_be_allocated(query.len() as u32) {
            let k = self.regulator.pattern_size;
            let max_pattern_count = v / k;
            self.spare_penalty_calculator.allocate(
                &self.regulator.penalties,
                self.regulator.cutoff.maximum_scaled_penalty_per_length,
                self.regulator.pattern_size,
                max_pattern_count,
            );
            self.wave_front_pool.allocate(
                v,
                &self.regulator.penalties,
                &self.regulator.cutoff,
            );
        }
        
        let reference_alignment_result = local_alignment_algorithm(
            reference,
            sequence_buffer,
            query,
            self.regulator.pattern_size,
            &self.regulator.penalties,
            &self.regulator.cutoff,
            &mut self.spare_penalty_calculator,
            &mut self.wave_front_pool.wave_front_1,
            &mut self.wave_front_pool.wave_front_2,
            &mut self.left_vpc_buffer,
            &mut self.right_vpc_buffer,
            &mut self.sorted_anchor_indices,
            &mut self.traversed_anchor_index_buffer,
            &mut self.operations_buffer,
            &mut self.extension_buffer,
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
