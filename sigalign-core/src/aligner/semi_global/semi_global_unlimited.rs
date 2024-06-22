use crate::results::AlignmentResult;
use crate::reference::{
    Reference, PatternIndex, SequenceStorage,
};
use crate::algorithm::semi_global_alignment_algorithm;
use super::{
    AlignmentRegulator,
    RegulatorError,
    SemiGlobalWorkspace,
};

#[derive(Clone)]
pub struct SemiGlobalAligner {
    pub(super) regulator: AlignmentRegulator,
    pub(super)workspace: SemiGlobalWorkspace,
}

impl SemiGlobalAligner {
    /// Create a new Aligner
    pub fn new(
        mismatch_penalty: u32,
        gap_open_penalty: u32,
        gap_extend_penalty: u32,
        minimum_alignment_length: u32,
        maximum_penalty_per_alignment_length: f32,
    ) -> Result<Self, RegulatorError> {
        let regulator = AlignmentRegulator::new_with_gcd_compressed(
            mismatch_penalty,
            gap_open_penalty,
            gap_extend_penalty,
            minimum_alignment_length,
            maximum_penalty_per_alignment_length,
        )?;
        let workspace = SemiGlobalWorkspace::init(&regulator);

        Ok(Self {
            regulator,
            workspace,
        })
    }
    /// Low-level alignment function
    #[inline]
    pub fn alignment<I: PatternIndex, S: SequenceStorage> (
        &mut self,
        // Query
        query: &[u8],
        // Targets
        reference: &Reference<I, S>,
        sequence_buffer: &mut S::Buffer,
        sorted_target_indices: &[u32],
    ) -> AlignmentResult {
        // Initialization
        self.workspace.allocate_more_space_if_needed(
            query.len() as u32,
            &self.regulator,
        );
        
        // Perform alignment
        let mut result = semi_global_alignment_algorithm(
            reference,
            sequence_buffer,
            query,
            sorted_target_indices,
            self.regulator.pattern_size,
            &self.regulator.penalties,
            &self.regulator.cutoff,
            &mut self.workspace.spare_penalty_calculator,
            &mut self.workspace.wave_front_buffer.as_mut(),
            &mut self.workspace.traversed_anchor_index_buffer,
            &mut self.workspace.operations_buffer,
            &mut self.workspace.extension_buffer,
        );
        self.regulator.decompress_result_with_gcd(&mut result);
        result
    }
}
