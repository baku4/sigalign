use crate::aligner::{
    Aligner,
    RegulatorError,
    allocation_strategy::LinearStrategy,
};

mod mode;
pub use mode::{
    SwitchableMode,
    ModeSwitchError,
};

pub type DefaultAligner = Aligner<SwitchableMode, LinearStrategy>;

impl DefaultAligner {
    pub fn new_local(
        mismatch_penalty: u32,
        gap_open_penalty: u32,
        gap_extend_penalty: u32,
        minimum_aligned_length: u32,
        maximum_penalty_per_length: f32,
    ) -> Result<Self, RegulatorError> {
        Self::new(mismatch_penalty, gap_open_penalty, gap_extend_penalty, minimum_aligned_length, maximum_penalty_per_length)
    }
    pub fn new_semi_global(
        mismatch_penalty: u32,
        gap_open_penalty: u32,
        gap_extend_penalty: u32,
        minimum_aligned_length: u32,
        maximum_penalty_per_length: f32,
    ) -> Result<Self, RegulatorError> {
        let mut v = Self::new(mismatch_penalty, gap_open_penalty, gap_extend_penalty, minimum_aligned_length, maximum_penalty_per_length)?;
        v.switch_to_semi_global().unwrap();
        Ok(v)
    }
    pub fn switch_to_semi_global(&mut self) -> Result<(), ModeSwitchError> {
        let query_length = self.query_length_checker.get_allocated_length();
        self.mode.switch_to_semi_global(
            query_length,
            &self.regulator,
        )
    }
    pub fn switch_to_local(&mut self) -> Result<(), ModeSwitchError> {
        let query_length = self.query_length_checker.get_allocated_length();
        self.mode.switch_to_local(
            query_length,
            &self.regulator,
        )
    }
    pub fn is_local_mode(&self) -> bool {
        self.mode.is_local_mode()
    }
}
