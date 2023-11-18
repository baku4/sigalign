use super::{Aligner, Algorithm, AllocationStrategy};
use std::fmt::Debug;

impl<A, L> Debug for Aligner<A, L> where
    A: Algorithm + Debug,
    L: AllocationStrategy,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Aligner")
            .field("algorithm", &self.algorithm)
            .field("length_checker", &self.query_length_checker)
            .field("regulator", &self.regulator)
            .finish()
    }
}

impl<A, L> Aligner<A, L> where
    A: Algorithm,
    L: AllocationStrategy,
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
