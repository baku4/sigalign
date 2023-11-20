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

pub trait MultipleSpaceManager {
    // Assuming regulators are sorted by cutoff.maximum_penalty_per_length
    //  - low to high = Strict to Lenient
    fn init(sorted_regulators: &[AlignmentRegulator]) -> Self;
    fn allocate_more_space_if_needed(
        &mut self,
        query_length: u32,
        most_lenient_regulator: &AlignmentRegulator,
    );
    fn reset(&mut self);
}

pub struct MultipleLocalSpaceManager<A: AllocationStrategy> {
    query_length_checker: QueryLengthChecker<A>,
    pub spare_penalty_calculators: Vec<SparePenaltyCalculator>,
    pub wave_front_pool: DoubleWaveFrontPool,
    pub left_vpc_buffer: Vec<Vpc>,
    pub right_vpc_buffer: Vec<Vpc>,
    pub traversed_anchor_index_buffer: Vec<AnchorIndex>,
    pub operations_buffer: Vec<AlignmentOperations>,
    pub extension_buffer: Vec<Extension>,
}

impl<A: AllocationStrategy> MultipleSpaceManager for MultipleLocalSpaceManager<A> {
    fn init(sorted_regulators: &[AlignmentRegulator]) -> Self {
        let allocation_strategy = A::new();
        let query_length_checker = QueryLengthChecker::new(allocation_strategy);
        let initial_query_length = query_length_checker.get_allocated_length();
        let spare_penalty_calculators = sorted_regulators.iter().map(|x| {
            SparePenaltyCalculator::new(
                &x.penalties,
                x.cutoff.maximum_scaled_penalty_per_length,
                x.pattern_size,
                initial_query_length / x.pattern_size,
            )
        }).collect();

        let most_lenient_regulator = sorted_regulators.last().unwrap();
        let wave_front_pool = DoubleWaveFrontPool::new(
            initial_query_length,
            most_lenient_regulator.cutoff.maximum_scaled_penalty_per_length,
            &most_lenient_regulator.penalties
        );
        Self {
            query_length_checker,
            spare_penalty_calculators,
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
        most_lenient_regulator: &AlignmentRegulator,
    ) {
        if let Some(required_query_length) = self.query_length_checker.optional_length_to_be_allocated(query_length) {
            let k = most_lenient_regulator.pattern_size;
            let max_pattern_count = required_query_length / k;
            self.spare_penalty_calculators.iter_mut().for_each(|x| {
                x.precalculate_right_spare_penalty(max_pattern_count);
            });

            self.wave_front_pool.allocate(
                required_query_length,
                most_lenient_regulator.cutoff.maximum_scaled_penalty_per_length,
                &most_lenient_regulator.penalties,
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

pub struct MultipleSemiGlobalSpaceManager<A: AllocationStrategy> {
    query_length_checker: QueryLengthChecker<A>,
    pub spare_penalty_calculators: Vec<SparePenaltyCalculator>,
    pub wave_front_pool: SingleWaveFrontPool,
    pub traversed_anchor_index_buffer: Vec<AnchorIndex>,
    pub operations_buffer: Vec<AlignmentOperations>,
    pub extension_buffer: Vec<Extension>,
}

impl<A: AllocationStrategy> MultipleSpaceManager for MultipleSemiGlobalSpaceManager<A> {
    fn init(sorted_regulators: &[AlignmentRegulator]) -> Self {
        let allocation_strategy = A::new();
        let query_length_checker = QueryLengthChecker::new(allocation_strategy);
        let initial_query_length = query_length_checker.get_allocated_length();
        let spare_penalty_calculators = sorted_regulators.iter().map(|x| {
            SparePenaltyCalculator::new(
                &x.penalties,
                x.cutoff.maximum_scaled_penalty_per_length,
                x.pattern_size,
                initial_query_length / x.pattern_size,
            )
        }).collect();

        let most_lenient_regulator = sorted_regulators.last().unwrap();
        let wave_front_pool = SingleWaveFrontPool::new(
            initial_query_length,
            most_lenient_regulator.cutoff.maximum_scaled_penalty_per_length,
            &most_lenient_regulator.penalties,
        );

        Self {
            query_length_checker,
            spare_penalty_calculators,
            wave_front_pool,
            traversed_anchor_index_buffer: Vec::new(),
            operations_buffer: Vec::new(),
            extension_buffer: Vec::new(),
        }
    }
    fn allocate_more_space_if_needed(
        &mut self,
        query_length: u32,
        most_lenient_regulator: &AlignmentRegulator,
    ) {
        if let Some(required_query_length) = self.query_length_checker.optional_length_to_be_allocated(query_length) {
            let k = most_lenient_regulator.pattern_size;
            let max_pattern_count = required_query_length / k;
            self.spare_penalty_calculators.iter_mut().for_each(|x| {
                x.precalculate_right_spare_penalty(max_pattern_count);
            });

            self.wave_front_pool.allocate(
                required_query_length,
                most_lenient_regulator.cutoff.maximum_scaled_penalty_per_length,
                &most_lenient_regulator.penalties,
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
