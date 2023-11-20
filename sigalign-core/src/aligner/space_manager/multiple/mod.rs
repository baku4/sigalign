use crate::{
    results::AlignmentOperations,
    algorithm::{
        AnchorIndex, Extension, SparePenaltyCalculator, Vpc,
    },
};
use super::{AlignmentRegulator, QueryLengthChecker, AllocationStrategy, DoubleWaveFrontPool};

pub trait MultipleCutoffsSpaceManager {
    fn init(regulator: &[AlignmentRegulator]) -> Self;
    fn allocate_more_space_if_needed(
        &mut self,
        query_length: u32,
        regulator: &AlignmentRegulator,
    );
    fn reset_buffers(&mut self);
}

pub struct LocalSpaceManager<A: AllocationStrategy> {
    query_length_checker: QueryLengthChecker<A>,
    pub spare_penalty_calculators: Vec<SparePenaltyCalculator>,
    pub wave_front_pool: DoubleWaveFrontPool,
    pub left_vpc_buffer: Vec<Vpc>,
    pub right_vpc_buffer: Vec<Vpc>,
    pub traversed_anchor_index_buffer: Vec<AnchorIndex>,
    pub operations_buffer: Vec<AlignmentOperations>,
    pub extension_buffer: Vec<Extension>,
}

impl<A: AllocationStrategy> LocalSpaceManager<A> {
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
            &regulator.penalties,
            &regulator.cutoff,
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
}