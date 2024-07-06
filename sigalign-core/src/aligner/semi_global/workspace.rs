use crate::{
    algorithm::{
        SparePenaltyCalculator, TraversedAnchor,
    },
    aligner::{
        regulator::AlignmentRegulator, workspace::{
            AllocationStrategy, DefaultDoublingStrategy, QueryLengthChecker, WaveFrontBuffer
        }
    },
    results::AlignmentOperations,
};

#[derive(Clone)]
pub struct SemiGlobalWorkspace {
    query_length_checker: QueryLengthChecker<DefaultDoublingStrategy>,
    pub spare_penalty_calculator: SparePenaltyCalculator,
    pub wave_front_buffer: WaveFrontBuffer,
    pub traversed_anchors_buffer: Vec<TraversedAnchor>,
    pub operations_buffer: Vec<AlignmentOperations>,
}
impl SemiGlobalWorkspace {
    pub fn init(regulator: &AlignmentRegulator) -> Self {
        let allocation_strategy = DefaultDoublingStrategy::new();
        let query_length_checker = QueryLengthChecker::new(allocation_strategy);
        let initial_query_length = query_length_checker.get_allocated_length();

        let spare_penalty_calculator = SparePenaltyCalculator::new(
            &regulator.penalties,
            regulator.cutoff.maximum_scaled_penalty_per_length,
            regulator.pattern_size,
            initial_query_length / regulator.pattern_size,
        );

        let wave_front_buffer = WaveFrontBuffer::new(initial_query_length, regulator.cutoff.maximum_scaled_penalty_per_length, &regulator.penalties);
        Self {
            query_length_checker,
            spare_penalty_calculator,
            wave_front_buffer,
            traversed_anchors_buffer: Vec::new(),
            operations_buffer: Vec::new(),
        }
    }
    pub fn allocate_more_space_if_needed(
        &mut self,
        query_length: u32,
        regulator: &AlignmentRegulator,
    ) {
        if let Some(required_query_length) = self.query_length_checker.optional_length_to_be_allocated(query_length) {
            let k = regulator.pattern_size;
            let max_pattern_count = required_query_length / k;
            self.spare_penalty_calculator.precalculate_right_spare_penalty(
                max_pattern_count,
            );

            self.wave_front_buffer.allocate(
                required_query_length,
                regulator.cutoff.maximum_scaled_penalty_per_length,
                &regulator.penalties,
            );
        }
    }
}
