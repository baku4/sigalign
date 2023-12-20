use thiserror::Error;

use sigalign_core::aligner::{
    AlignmentRegulator, RegulatorError,
};
use super::{Aligner, DynamicAligner};

const MINIMUM_PATTERN_SIZE: u32 = 4;

/// Error for building `Aligner`.
#[derive(Debug, Error)]
pub enum AlignerBuildError {
    #[error("Invalid regulator: {0}")]
    InvalidRegulator(#[from] RegulatorError),
    #[error("Cutoff is too low to detect the pattern.")]
    LowCutoff,
}

impl Aligner {
    /// Make a new `Aligner`.
    pub fn new(
        mismatch_penalty: u32,
        gap_open_penalty: u32,
        gap_extend_penalty: u32,
        min_length: u32,
        max_penalty_per_length: f32,
    ) -> Result<Self, AlignerBuildError> {
        let regulator = AlignmentRegulator::new(
            mismatch_penalty,
            gap_open_penalty,
            gap_extend_penalty,
            min_length,
            max_penalty_per_length,
        )?;
        if regulator.get_pattern_size() < MINIMUM_PATTERN_SIZE {
            return Err(AlignerBuildError::LowCutoff);
        }

        let dynamic_aligner = DynamicAligner::new_local(regulator.clone());

        Ok(Self {
            regulator,
            dynamic_aligner,
        })
    }
}
