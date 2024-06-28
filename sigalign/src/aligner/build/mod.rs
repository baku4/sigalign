use sigalign_core::aligner::{AlignmentRegulator, RegulatorError};
use sigalign_impl::sequence_storage::in_memory::InMemoryBuffer;
use super::{Aligner, DynamicAligner};

pub enum Algorithm {
    Local,
    SemiGlobal,
}

const MINIMUM_PATTERN_SIZE: u32 = 4;
#[derive(Debug, thiserror::Error)]
pub enum AlignerBuildError {
    #[error("Invalid regulator: {0}")]
    InvalidRegulator(#[from] RegulatorError),
    #[error("Cutoff is too low to detect the pattern.")]
    LowCutoff,
}

impl Aligner {
    pub fn new(
        algorithm: Algorithm,
        mismatch_penalty: u32,
        gap_open_penalty: u32,
        gap_extend_penalty: u32,
        min_length: u32,
        max_penalty_per_length: f32,
    ) -> Result<Self, AlignerBuildError> {
        match algorithm {
            Algorithm::Local => Self::new_local(
                mismatch_penalty,
                gap_open_penalty,
                gap_extend_penalty,
                min_length,
                max_penalty_per_length,
            ),
            Algorithm::SemiGlobal => Self::new_semi_global(
                mismatch_penalty,
                gap_open_penalty,
                gap_extend_penalty,
                min_length,
                max_penalty_per_length,
            ),
        }
    }
    pub fn new_local(
        mismatch_penalty: u32,
        gap_open_penalty: u32,
        gap_extend_penalty: u32,
        min_length: u32,
        max_penalty_per_length: f32,
    ) -> Result<Self, AlignerBuildError> {
        let dynamic_aligner = DynamicAligner::new_local(
            AlignmentRegulator::new(
                mismatch_penalty,
                gap_open_penalty,
                gap_extend_penalty,
                min_length,
                max_penalty_per_length,
            )?
        );
        Self::from_dynamic_aligner(dynamic_aligner)
    }
    pub fn new_semi_global(
        mismatch_penalty: u32,
        gap_open_penalty: u32,
        gap_extend_penalty: u32,
        min_length: u32,
        max_penalty_per_length: f32,
    ) -> Result<Self, AlignerBuildError> {
        let dynamic_aligner = DynamicAligner::new_semi_global(
            AlignmentRegulator::new(
                mismatch_penalty,
                gap_open_penalty,
                gap_extend_penalty,
                min_length,
                max_penalty_per_length,
            )?
        );
        Self::from_dynamic_aligner(dynamic_aligner)
    }
    fn from_dynamic_aligner(dynamic_aligner: DynamicAligner) -> Result<Self, AlignerBuildError> {
        if dynamic_aligner.regulator().get_pattern_size() < MINIMUM_PATTERN_SIZE {
            return Err(AlignerBuildError::LowCutoff);
        }
        Ok(Self {
            dynamic_aligner,
            sequence_buffer: InMemoryBuffer::new(),
        })
    }
}
