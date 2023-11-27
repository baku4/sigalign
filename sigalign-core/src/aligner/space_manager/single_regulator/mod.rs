use crate::{
    results::AlignmentOperations,
    algorithm::{
        AnchorIndex, Extension, SparePenaltyCalculator, Vpc,
    },
};
use super::{
    AlignmentRegulator,
    QueryLengthChecker,
    AllocationStrategy,
    WaveFrontPool,
    SingleWaveFrontPool,
    DoubleWaveFrontPool,
};

pub trait SingleSpaceManager {
    fn init(regulator: &AlignmentRegulator) -> Self;
    fn allocate_more_space_if_needed(
        &mut self,
        query_length: u32,
        regulator: &AlignmentRegulator,
    );
    fn reset(&mut self);
}

#[derive(Clone)]
pub struct SingleLocalSpaceManager<A: AllocationStrategy> {
    query_length_checker: QueryLengthChecker<A>,
    pub spare_penalty_calculator: SparePenaltyCalculator,
    pub wave_front_pool: DoubleWaveFrontPool,
    pub left_vpc_buffer: Vec<Vpc>,
    pub right_vpc_buffer: Vec<Vpc>,
    pub traversed_anchor_index_buffer: Vec<AnchorIndex>,
    pub operations_buffer: Vec<AlignmentOperations>,
    pub extension_buffer: Vec<Extension>,
}

impl<A: AllocationStrategy> SingleSpaceManager for SingleLocalSpaceManager<A> {
    fn init(regulator: &AlignmentRegulator) -> Self {
        let allocation_strategy = A::new();
        let query_length_checker = QueryLengthChecker::new(allocation_strategy);
        let initial_query_length = query_length_checker.get_allocated_length();
        let spare_penalty_calculator = SparePenaltyCalculator::new(
            &regulator.penalties,
            regulator.cutoff.maximum_scaled_penalty_per_length,
            regulator.pattern_size,
            initial_query_length / regulator.pattern_size,
        );

        let wave_front_pool = DoubleWaveFrontPool::new(
            initial_query_length,
            regulator.cutoff.maximum_scaled_penalty_per_length,
            &regulator.penalties,
        );
        Self {
            query_length_checker,
            spare_penalty_calculator,
            wave_front_pool,
            left_vpc_buffer: Vec::new(),
            right_vpc_buffer: Vec::new(),
            traversed_anchor_index_buffer: Vec::new(),
            operations_buffer: Vec::new(),
            extension_buffer: Vec::new(),
        }
    }
    fn allocate_more_space_if_needed(
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

            self.wave_front_pool.allocate(
                required_query_length,
                regulator.cutoff.maximum_scaled_penalty_per_length,
                &regulator.penalties,
            );
        }
    }
    fn reset(
        &mut self,
    ) {
        self.left_vpc_buffer = Vec::new();
        self.right_vpc_buffer = Vec::new();
        self.traversed_anchor_index_buffer = Vec::new();
        self.operations_buffer = Vec::new();
        self.extension_buffer = Vec::new();
    }
}

#[derive(Clone)]
pub struct SingleSemiGlobalSpaceManager<A: AllocationStrategy> {
    query_length_checker: QueryLengthChecker<A>,
    pub spare_penalty_calculator: SparePenaltyCalculator,
    pub wave_front_pool: SingleWaveFrontPool,
    pub traversed_anchor_index_buffer: Vec<AnchorIndex>,
    pub operations_buffer: Vec<AlignmentOperations>,
    pub extension_buffer: Vec<Extension>,
}

impl<A: AllocationStrategy> SingleSpaceManager for SingleSemiGlobalSpaceManager<A> {
    fn init(regulator: &AlignmentRegulator) -> Self {
        let allocation_strategy = A::new();
        let query_length_checker = QueryLengthChecker::new(allocation_strategy);
        let initial_query_length = query_length_checker.get_allocated_length();
        let spare_penalty_calculator = SparePenaltyCalculator::new(
            &regulator.penalties,
            regulator.cutoff.maximum_scaled_penalty_per_length,
            regulator.pattern_size,
            initial_query_length / regulator.pattern_size,
        );
        let wave_front_pool = SingleWaveFrontPool::new(
            initial_query_length,
            regulator.cutoff.maximum_scaled_penalty_per_length,
            &regulator.penalties,
        );

        Self {
            query_length_checker,
            spare_penalty_calculator,
            wave_front_pool,
            traversed_anchor_index_buffer: Vec::new(),
            operations_buffer: Vec::new(),
            extension_buffer: Vec::new(),
        }
    }
    fn allocate_more_space_if_needed(
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

            self.wave_front_pool.allocate(
                required_query_length,
                regulator.cutoff.maximum_scaled_penalty_per_length,
                &regulator.penalties,
            );
        }
    }
    fn reset(
        &mut self,
    ) {
        self.traversed_anchor_index_buffer = Vec::new();
        self.operations_buffer = Vec::new();
        self.extension_buffer = Vec::new();
    }
}
