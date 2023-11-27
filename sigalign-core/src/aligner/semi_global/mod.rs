use crate::{
    results::AlignmentResult,
    reference::{
        Reference, PatternIndex, SequenceStorage,
    },
    algorithm::semi_global_alignment_algorithm,
};
use super::{
    Aligner,
    AlignmentRegulator,
    SingleSpaceManager,
    SingleSemiGlobalSpaceManager,
    AllocationStrategy,
};

#[derive(Clone)]
pub struct SemiGlobalAligner<A: AllocationStrategy> {
    regulator: AlignmentRegulator,
    space_manager: SingleSemiGlobalSpaceManager<A>,
}

impl<A: AllocationStrategy> Aligner for SemiGlobalAligner<A> {
    fn alignment<I: PatternIndex, S: SequenceStorage> (
        &mut self,
        reference: &Reference<I, S>,
        sequence_buffer: &mut S::Buffer,
        sorted_target_indices: &[u32],
        query: &[u8],
    ) -> AlignmentResult {
        self.space_manager.allocate_more_space_if_needed(query.len() as u32, &self.regulator);

        let mut result = semi_global_alignment_algorithm(
            reference,
            sequence_buffer,
            query,
            sorted_target_indices,
            self.regulator.pattern_size,
            &self.regulator.penalties,
            &self.regulator.cutoff,
            &mut self.space_manager.spare_penalty_calculator,
            &mut self.space_manager.wave_front_pool.wave_front,
            &mut self.space_manager.traversed_anchor_index_buffer,
            &mut self.space_manager.operations_buffer,
            &mut self.space_manager.extension_buffer,
        );
        result.multiply_gcd(self.regulator.gcd_for_compression);
        result
    }
}

impl<A: AllocationStrategy> SemiGlobalAligner<A> {
    pub fn new(
        regulator: AlignmentRegulator,
    ) -> Self {
        let space_manager = SingleSemiGlobalSpaceManager::init(&regulator);
        Self {
            regulator,
            space_manager,
        }
    }
    pub fn get_regulator(&self) -> &AlignmentRegulator {
        &self.regulator
    }
}
