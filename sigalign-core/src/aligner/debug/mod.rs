use super::{Aligner, AlignmentMode, AllocationStrategy};
use std::fmt::Debug;

impl<M, A> Debug for Aligner<M, A> where
    M: AlignmentMode + Debug,
    A: AllocationStrategy,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Aligner")
            .field("mode", &self.mode)
            .field("length_checker", &self.query_length_checker)
            .field("regulator", &self.regulator)
            .finish()
    }
}

impl<M, A> Aligner<M, A> where
    M: AlignmentMode,
    A: AllocationStrategy,
{
    /// Get mismatch penalty
    pub fn get_mismatch_penalty(&self) -> u32 {
        self.regulator.get_mismatch_penalty()
    }
    /// Get gap-open penalty
    pub fn get_gap_open_penalty(&self) -> u32 {
        self.regulator.get_gap_open_penalty()
    }
    /// Get gap-extend penalty
    pub fn get_gap_extend_penalty(&self) -> u32 {
        self.regulator.get_gap_extend_penalty()
    }
    /// Get minimum aligned length
    pub fn get_minimum_aligned_length(&self) -> u32 {
        self.regulator.get_minimum_aligned_length()
    }
    /// Get maximum penalty per length
    pub fn get_maximum_penalty_per_length(&self) -> f32 {
        self.regulator.get_maximum_penalty_per_length()
    }
    /// Get size of pattern
    pub fn get_pattern_size(&self) -> u32 {
        self.regulator.get_pattern_size()
    }
}
