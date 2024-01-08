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
        self.space_manager.sorted_target_indices_buffer.clear();
        self.space_manager.sorted_target_indices_buffer.extend_from_slice(sorted_target_indices);
        let mut target_alignment_results = Vec::new();
        
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
            alignment_result.0.into_iter().for_each(|mut target_alignment_result| {
                target_alignment_result.multiply_gcd(regulator.gcd_for_compression);
                // Remove the target index from the buffer
                let index_of_target_index = self.space_manager.sorted_target_indices_buffer.binary_search(&target_alignment_result.index).unwrap();
                self.space_manager.sorted_target_indices_buffer.remove(index_of_target_index);
                target_alignment_results.push(target_alignment_result);
            });
        }

        AlignmentResult(target_alignment_results)
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
