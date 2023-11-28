use crate::{
    results::AlignmentResult,
    reference::{
        Reference, PatternIndex, SequenceStorage,
    },
    algorithm::local_alignment_algorithm_with_limit,
};
use super::{
    Aligner,
    AlignmentRegulator,
    SingleSpaceManager,
    SingleLocalSpaceManager,
    AllocationStrategy,
};

#[derive(Clone)]
pub struct LocalWithLimitAligner<A: AllocationStrategy> {
    pub(super) regulator: AlignmentRegulator,
    pub(super) space_manager: SingleLocalSpaceManager<A>,
    pub(super) limit: u32,
}

impl<A: AllocationStrategy> Aligner for LocalWithLimitAligner<A> {
    fn alignment<I: PatternIndex, S: SequenceStorage> (
        &mut self,
        reference: &Reference<I, S>,
        sequence_buffer: &mut S::Buffer,
        sorted_target_indices: &[u32],
        query: &[u8],
    ) -> AlignmentResult {
        self.space_manager.allocate_more_space_if_needed(query.len() as u32, &self.regulator);

        let mut result = local_alignment_algorithm_with_limit(
            reference,
            sequence_buffer,
            query,
            sorted_target_indices,
            self.regulator.pattern_size,
            &self.regulator.penalties,
            &self.regulator.cutoff,
            &mut self.space_manager.spare_penalty_calculator,
            &mut self.space_manager.wave_front_pool.wave_front_1,
            &mut self.space_manager.wave_front_pool.wave_front_2,
            &mut self.space_manager.left_vpc_buffer,
            &mut self.space_manager.right_vpc_buffer,
            &mut self.space_manager.traversed_anchor_index_buffer,
            &mut self.space_manager.operations_buffer,
            &mut self.space_manager.extension_buffer,
            self.limit,
        );
        result.multiply_gcd(self.regulator.gcd_for_compression);
        result
    }
}

impl<A: AllocationStrategy> LocalWithLimitAligner<A> {
    pub fn new(
        regulator: AlignmentRegulator,
        limit: u32,
    ) -> Self {
        let space_manager = SingleLocalSpaceManager::init(&regulator);
        Self {
            regulator,
            space_manager,
            limit,
        }
    }
    pub fn get_regulator(&self) -> &AlignmentRegulator {
        &self.regulator
    }
    pub fn get_limit(&self) -> u32 {
        self.limit
    }
    pub fn set_limit(&mut self, limit: u32) {
        self.limit = limit;
    }
}
