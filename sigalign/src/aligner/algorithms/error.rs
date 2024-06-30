use sigalign_core::aligner::{AlignmentRegulator, RegulatorError};

/// Errors related to defining alignment algorithms.
#[derive(Debug, thiserror::Error)]
pub enum ParamsError {
    #[error("Invalid input value: {0}")]
    InvalidValue(String),
    #[error("Operation inhibited due to low efficiency: {0}")]
    InhibitedLowEfficiency(String),
}

impl From::<RegulatorError> for ParamsError  {
    fn from(value: RegulatorError) -> Self {
        let err_msg = value.to_string();
        Self::InvalidValue(err_msg)
    }
}

const MINIMUM_PATTERN_SIZE: u32 = 4;
pub fn check_pattern_size(alignment_regulator: &AlignmentRegulator) -> Result<(), ParamsError> {
    if alignment_regulator.get_pattern_size() < MINIMUM_PATTERN_SIZE {
        Err(ParamsError::InhibitedLowEfficiency(
            "Cutoff is too low to detect the pattern.".to_string(),
        ))
    } else {
        Ok(())
    }
}
