use crate::aligner::{
    LocalAligner, SemiGlobalAligner, AlignerInterface, RegulatorError,
    AllocationStrategy, LinearStrategy, DoublingStrategy,
};

#[derive(Clone)]
pub struct DefaultAligner {
    inner: SelfDescAligner,
}
#[derive(Clone)]
enum SelfDescAligner {
    Local(LocalAligner<LinearStrategy>),
    SemiGlobal(SemiGlobalAligner<LinearStrategy>),
}

impl DefaultAligner {
    pub fn new_local(
        mismatch_penalty: u32,
        gap_open_penalty: u32,
        gap_extend_penalty: u32,
        minimum_aligned_length: u32,
        maximum_penalty_per_length: f32,
    ) -> Result<Self, RegulatorError> {
        let aligner = LocalAligner::new(mismatch_penalty, gap_open_penalty, gap_extend_penalty, minimum_aligned_length, maximum_penalty_per_length)?;
        Ok(Self { inner: SelfDescAligner::Local(aligner) })
    }
    pub fn new_semi_global(
        mismatch_penalty: u32,
        gap_open_penalty: u32,
        gap_extend_penalty: u32,
        minimum_aligned_length: u32,
        maximum_penalty_per_length: f32,
    ) -> Result<Self, RegulatorError> {
        let aligner = SemiGlobalAligner::new(mismatch_penalty, gap_open_penalty, gap_extend_penalty, minimum_aligned_length, maximum_penalty_per_length)?;
        Ok(Self { inner: SelfDescAligner::SemiGlobal(aligner) })
    }
}

mod alignment;
pub use alignment::DefaultAlignmentError;

mod debug;