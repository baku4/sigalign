use crate::results::AlignmentResult;
use crate::reference::{
    Reference,
    pattern_index::PatternIndex,
    sequence_storage::SequenceStorage,
};
use crate::aligner::{
    AlignmentRegulator,
    mode::{AlignmentMode, LocalMode, SemiGlobalMode},
};

#[derive(Debug)]
pub enum SwitchableMode {
    Local(LocalMode),
    SemiGlobal(SemiGlobalMode),
}

impl AlignmentMode for SwitchableMode {
    fn new(
        initial_query_length: u32,
        regulator: &AlignmentRegulator,
    ) -> Self {
        Self::Local(LocalMode::new(initial_query_length, regulator))
    }
    fn allocate_space(
        &mut self,
        required_query_length: u32,
        regulator: &AlignmentRegulator,
    ) {
        match self {
            Self::Local(v) => v.allocate_space(required_query_length, regulator),
            Self::SemiGlobal(v) => v.allocate_space(required_query_length, regulator),
        }
    }
    fn reset_buffers(
        &mut self,
    ) {
        match self {
            Self::Local(v) => v.reset_buffers(),
            Self::SemiGlobal(v) => v.reset_buffers(),
        }
    }
    fn run_algorithm<I: PatternIndex, S: SequenceStorage>(
        &mut self,
        reference: &Reference<I, S>,
        sequence_buffer: &mut S::Buffer,
        query: &[u8],
        regulator: &AlignmentRegulator,
    ) -> AlignmentResult {
        match self {
            Self::Local(v) => v.run_algorithm(
                reference,
                sequence_buffer,
                query,
                regulator,
            ),
            Self::SemiGlobal(v) => v.run_algorithm(
                reference,
                sequence_buffer,
                query,
                regulator,
            ),
        }
    }
}

impl SwitchableMode {
    pub fn new_local(
        initial_query_length: u32,
        regulator: &AlignmentRegulator,
    ) -> Self {
        Self::Local(LocalMode::new(initial_query_length, regulator))
    }
    pub fn new_semi_global(
        initial_query_length: u32,
        regulator: &AlignmentRegulator,
    ) -> Self {
        Self::SemiGlobal(SemiGlobalMode::new(initial_query_length, regulator))
    }
    pub fn is_local_mode(&self) -> bool {
        match self {
            Self::Local(_) => true,
            Self::SemiGlobal(_) => false,
        }
    }
    pub fn switch_to_semi_global(
        &mut self,
        initial_query_length: u32,
        regulator: &AlignmentRegulator,
    ) -> Result<(), ModeSwitchError> {
        match self {
            Self::SemiGlobal(_) => {
                Err(ModeSwitchError::SameMode)
            },
            _ => {
                *self = Self::new_semi_global(initial_query_length, regulator);
                Ok(())
            }
        }
    }
    pub fn switch_to_local(
        &mut self,
        initial_query_length: u32,
        regulator: &AlignmentRegulator,
    ) -> Result<(), ModeSwitchError> {
        match self {
            Self::Local(_) => {
                Err(ModeSwitchError::SameMode)
            },
            _ => {
                *self = Self::new_local(initial_query_length, regulator);
                Ok(())
            }
        }
    }
}

use thiserror::Error;
#[derive(Error, Debug)]
pub enum ModeSwitchError {
    #[error("Cannot switch to the same mode.")]
    SameMode,
}