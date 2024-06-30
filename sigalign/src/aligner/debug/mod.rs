use std::fmt::Debug;

use super::{
    Aligner,
    algorithms::Algorithm,
};

impl<A: Algorithm> Debug for Aligner<A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {    
        f.debug_struct("Aligner")
            .field("algorithm", &self.algorithm)
            .field("sequence_buffer", &"InMemorySequenceBuffer")
            .finish()
    }
}

impl<A: Algorithm> Aligner<A> {
    /// Get mismatch penalty
    pub fn get_mismatch_penalty(&self) -> u32 {
        self.algorithm.regulator().get_mismatch_penalty()
    }
    /// Get gap-open penalty
    pub fn get_gap_open_penalty(&self) -> u32 {
        self.algorithm.regulator().get_gap_open_penalty()
    }
    /// Get gap-extend penalty
    pub fn get_gap_extend_penalty(&self) -> u32 {
        self.algorithm.regulator().get_gap_extend_penalty()
    }
    /// Get minimum length
    pub fn get_minimum_length(&self) -> u32 {
        self.algorithm.regulator().get_minimum_length()
    }
    /// Get maximum penalty per length
    pub fn get_maximum_penalty_per_length(&self) -> f32 {
        self.algorithm.regulator().get_maximum_penalty_per_length()
    }
    /// Get size of pattern
    pub fn get_pattern_size(&self) -> u32 {
        self.algorithm.regulator().get_pattern_size()
    }
}
