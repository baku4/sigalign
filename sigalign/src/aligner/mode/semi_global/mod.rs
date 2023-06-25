use crate::algorithm::{
    // Algorithms
    semi_global_alignment_algorithm,
    // Structs to be buffered
    //   - common
    AnchorIndex, Extension, SparePenaltyCalculator,
};
use crate::results::{
    AlignmentResult,
    AlignmentOperations,
};
use crate::reference::{
    Reference,
    pattern_index::PatternIndex,
    sequence_storage::SequenceStorage,
};
use super::{
    AlignmentMode,
    AlignmentRegulator,
    WaveFrontPool, SingleWaveFrontPool,
};

pub struct SemiGlobalMode {
    spare_penalty_calculator: SparePenaltyCalculator,
    wave_front_pool: SingleWaveFrontPool,
    // Buffers
    traversed_anchor_index_buffer: Vec<AnchorIndex>,
    operations_buffer: Vec<AlignmentOperations>,
    extension_buffer: Vec<Extension>,
}
impl AlignmentMode for SemiGlobalMode {
    fn new(
        initial_query_length: u32,
        regulator: &AlignmentRegulator,
    ) -> Self {
        let spare_penalty_calculator = SparePenaltyCalculator::new(
            &regulator.penalties,
            regulator.cutoff.maximum_scaled_penalty_per_length,
            regulator.pattern_size,
            initial_query_length / regulator.pattern_size,
        );
        let wave_front_pool = SingleWaveFrontPool::new(
            initial_query_length,
            &regulator.penalties,
            &regulator.cutoff,
        );
        Self {
            spare_penalty_calculator,
            wave_front_pool,
            traversed_anchor_index_buffer: Vec::new(),
            operations_buffer: Vec::new(),
            extension_buffer: Vec::new(),
        }

    }
    fn allocate_space(
        &mut self,
        required_query_length: u32,
        regulator: &AlignmentRegulator,
    ) {
        let k = regulator.pattern_size;
        let max_pattern_count = required_query_length / k;
        self.spare_penalty_calculator.precalculate_right_spare_penalty(
            max_pattern_count,
        );

        self.wave_front_pool.allocate(
            required_query_length,
            &regulator.penalties,
            &regulator.cutoff,
        );
    }
    fn reset_buffers(
        &mut self,
    ) {
        self.traversed_anchor_index_buffer = Vec::new();
        self.operations_buffer = Vec::new();
        self.extension_buffer = Vec::new();
    }
    fn run_algorithm<I: PatternIndex, S: SequenceStorage>(
        &mut self,
        reference: &Reference<I, S>,
        sequence_buffer: &mut S::Buffer,
        query: &[u8],
        regulator: &AlignmentRegulator,
    ) -> AlignmentResult {
        semi_global_alignment_algorithm(
            reference,
            sequence_buffer,
            query,
            regulator.pattern_size,
            &regulator.penalties,
            &regulator.cutoff,
            &mut self.spare_penalty_calculator,
            &mut self.wave_front_pool.wave_front,
            &mut self.traversed_anchor_index_buffer,
            &mut self.operations_buffer,
            &mut self.extension_buffer,
        )
    }
}

impl std::fmt::Debug for SemiGlobalMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SemiGlobalMode")
    }
}