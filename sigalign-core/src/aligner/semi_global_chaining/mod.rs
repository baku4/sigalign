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
    MultipleSpaceManager,
    MultipleSemiGlobalSpaceManager,
    AllocationStrategy,
};

pub struct SemiGlobalChainingAligner<A: AllocationStrategy> {
    sorted_regulators: Vec<AlignmentRegulator>,
    space_manager: MultipleSemiGlobalSpaceManager<A>,
}

impl<A: AllocationStrategy> Aligner for SemiGlobalChainingAligner<A> {
    fn alignment<I: PatternIndex, S: SequenceStorage> (
        &mut self,
        reference: &Reference<I, S>,
        sequence_buffer: &mut S::Buffer,
        sorted_target_indices: &[u32],
        query: &[u8],
    ) -> AlignmentResult {
        let most_lenient_regulator = self.sorted_regulators.last().unwrap();
        self.space_manager.allocate_more_space_if_needed(query.len() as u32, most_lenient_regulator);
        
        for (index, regulator) in self.sorted_regulators.iter().enumerate() {
            let alignment_result = semi_global_alignment_algorithm(
                reference,
                sequence_buffer,
                query,
                sorted_target_indices,
                regulator.pattern_size,
                &regulator.penalties,
                &regulator.cutoff,
                &mut self.space_manager.spare_penalty_calculators[index],
                &mut self.space_manager.wave_front_pool.wave_front,
                &mut self.space_manager.traversed_anchor_index_buffer,
                &mut self.space_manager.operations_buffer,
                &mut self.space_manager.extension_buffer,
            );
            if !alignment_result.0.is_empty() {
                return alignment_result
            }
        }

        AlignmentResult(Vec::new())
    }
}

impl<A: AllocationStrategy> SemiGlobalChainingAligner<A> {
    pub fn new(sorted_regulators: Vec<AlignmentRegulator>) -> Self {
        let space_manager = MultipleSemiGlobalSpaceManager::init(&sorted_regulators);
        Self {
            sorted_regulators,
            space_manager,
        }
    }
    pub fn get_regulators(&self) -> &[AlignmentRegulator] {
        &self.sorted_regulators
    }
}
