use thiserror::Error;

use sigalign_core::aligner::{
        AlignmentRegulator, RegulatorError,
    };

mod dynamic_aligner;
use dynamic_aligner::DynamicAligner;

mod perform_alignments;
mod switch_algorithm;

mod debug;

const MINIMUM_PATTERN_SIZE: u32 = 4;

/// An alignment executor.
#[derive(Clone)]
pub struct Aligner {
    regulator: AlignmentRegulator,
    dynamic_aligner: DynamicAligner,
}

impl Aligner {
    /* Make new Aligner */
    /// Make a new `Aligner`.
    pub fn new(
        mismatch_penalty: u32,
        gap_open_penalty: u32,
        gap_extend_penalty: u32,
        min_length: u32,
        max_penalty_per_length: f32,
        is_local: bool,
        limit: Option<u32>,
    ) -> Result<Self, AlignerBuildError> {
        let regulator = Self::get_regulator(mismatch_penalty, gap_open_penalty, gap_extend_penalty, min_length, max_penalty_per_length)?;
        if regulator.get_pattern_size() < MINIMUM_PATTERN_SIZE {
            return Err(AlignerBuildError::LowCutoff);
        }

        let dynamic_aligner = match limit {
            None => {
                if is_local {
                    DynamicAligner::new_local(regulator.clone())
                } else {
                    DynamicAligner::new_semi_global(regulator.clone())
                }
            },
            Some(limit) => {
                if is_local {
                    DynamicAligner::new_local_with_limit(regulator.clone(), limit)
                } else {
                    DynamicAligner::new_semi_global_with_limit(regulator.clone(), limit)
                }
            },
        };

        Ok(Self {
            regulator,
            dynamic_aligner,
        })
    }
    fn get_regulator(
        mismatch_penalty: u32,
        gap_open_penalty: u32,
        gap_extend_penalty: u32,
        min_length: u32,
        max_penalty_per_length: f32,
    ) -> Result<AlignmentRegulator, AlignerBuildError> {
        let regulator = AlignmentRegulator::new(
            mismatch_penalty,
            gap_open_penalty,
            gap_extend_penalty,
            min_length,
            max_penalty_per_length,
        )?;
        Ok(regulator)
    }
}

/// Error for building `Aligner`.
#[derive(Debug, Error)]
pub enum AlignerBuildError {
    #[error("Invalid regulator: {0}")]
    InvalidRegulator(#[from] RegulatorError),
    #[error("Cutoff is too low to detect the pattern.")]
    LowCutoff,
}
