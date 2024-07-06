use crate::results::QueryAlignment;
use crate::reference::{
    Reference, PatternIndex, SequenceStorage,
};
use crate::algorithm::local_alignment_algorithm;
use super::{
    AlignmentRegulator,
    LocalWorkspace,
};

#[derive(Clone)]
pub struct LocalAligner {
    pub(super) regulator: AlignmentRegulator,
    pub(super) workspace: LocalWorkspace,
}

impl LocalAligner {
    /// Create a new Aligner from alignment regulator
    pub fn new(regulator: AlignmentRegulator) -> Self {
        let workspace = LocalWorkspace::init(&regulator);
        Self {
            regulator,
            workspace,
        }
    }
    /// Low-level alignment function
    #[inline]
    pub fn align<I: PatternIndex, S: SequenceStorage> (
        &mut self,
        // Query
        query: &[u8],
        // Targets
        reference: &Reference<I, S>,
        sequence_buffer: &mut S::Buffer,
        sorted_target_indices: &[u32],
    ) -> QueryAlignment {
        // Initialization
        self.workspace.allocate_more_space_if_needed(
            query.len() as u32,
            &self.regulator,
        );
        
        // Perform alignment
        let mut result = local_alignment_algorithm(
            reference,
            sequence_buffer,
            query,
            sorted_target_indices,
            self.regulator.pattern_size,
            &self.regulator.penalties,
            &self.regulator.cutoff,
            &mut self.workspace.spare_penalty_calculator,
            &mut self.workspace.wave_front_buffer_1.as_mut(),
            &mut self.workspace.wave_front_buffer_2.as_mut(),
            &mut self.workspace.left_vpc_buffer,
            &mut self.workspace.right_vpc_buffer,
            &mut self.workspace.traversed_anchors_buffer,
            &mut self.workspace.operations_buffer,
        );
        self.regulator.decompress_result_with_gcd(&mut result);
        result
    }
    pub fn regulator(&self) -> &AlignmentRegulator {
        &self.regulator
    }
}
