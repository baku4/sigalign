use std::fmt::Debug;

use sigalign_core::aligner::AlignmentRegulator;

use super::{
    Aligner, DynamicAligner,
};

impl Debug for Aligner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Aligner")
            .field("algorithm", &self.dynamic_aligner.algorithm_string())
            .field("regulator", &self.dynamic_aligner.regulator())
            .finish()
    }
}

impl DynamicAligner {
    fn algorithm_string(&self) -> String {
        match self {
            Self::Local(_) => "Local".to_string(),
            Self::LocalWithLimit(v) => {
                let limit = v.limit();
                format!("LocalWithLimit({})", limit)
            },
            Self::SemiGlobal(_) => "SemiGlobal".to_string(),
            Self::SemiGlobalWithLimit(v) => {
                let limit = v.limit();
                format!("SemiGlobalWithLimit({})", limit)
            },
        }
    }
}

impl Aligner {
    /// Get mismatch penalty
    pub fn get_mismatch_penalty(&self) -> u32 {
        self.regulator().get_mismatch_penalty()
    }
    /// Get gap-open penalty
    pub fn get_gap_open_penalty(&self) -> u32 {
        self.regulator().get_gap_open_penalty()
    }
    /// Get gap-extend penalty
    pub fn get_gap_extend_penalty(&self) -> u32 {
        self.regulator().get_gap_extend_penalty()
    }
    /// Get minimum aligned length
    pub fn get_minimum_aligned_length(&self) -> u32 {
        self.regulator().get_minimum_aligned_length()
    }
    /// Get maximum penalty per length
    pub fn get_maximum_penalty_per_length(&self) -> f32 {
        self.regulator().get_maximum_penalty_per_length()
    }
    /// Get size of pattern
    pub fn get_pattern_size(&self) -> u32 {
        self.regulator().get_pattern_size()
    }
    fn regulator(&self) -> &AlignmentRegulator {
        self.dynamic_aligner.regulator()
    }
}
