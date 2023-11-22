use crate::results::AlignmentResult;
use crate::reference::{
    Reference,
    pattern_index::PatternIndex,
    sequence_storage::SequenceStorage,
};

// Internal structures for Aligner
mod regulator;
pub(crate) use regulator::AlignmentRegulator;
pub use regulator::RegulatorError;
pub mod allocation_strategy;
use allocation_strategy::{
    QueryLengthChecker,
    AllocationStrategy,
};
pub mod mode;
use mode::AlignmentMode;

#[derive(Clone)]
pub struct Aligner<M, A> where
    M: AlignmentMode,
    A: AllocationStrategy,
{
    pub(crate) regulator: AlignmentRegulator,
    pub(crate) query_length_checker: QueryLengthChecker<A>,
    pub(crate) mode: M,
}

impl<M, A> Aligner<M, A> where
    M: AlignmentMode,
    A: AllocationStrategy,
{
    pub fn new(
        mismatch_penalty: u32,
        gap_open_penalty: u32,
        gap_extend_penalty: u32,
        minimum_aligned_length: u32,
        maximum_penalty_per_length: f32,
    ) -> Result<Self, RegulatorError> {
        let regulator = AlignmentRegulator::new(mismatch_penalty, gap_open_penalty, gap_extend_penalty, minimum_aligned_length, maximum_penalty_per_length)?;
        let query_length_checker = QueryLengthChecker::new();
        let query_length = query_length_checker.get_allocated_length();

        let mode = M::new(
            query_length,
            &regulator,
        );

        Ok(Self {
            regulator,
            query_length_checker,
            mode,
        })
    }
    pub fn alignment<I: PatternIndex, S: SequenceStorage> (
        &mut self,
        reference: &Reference<I, S>,
        sequence_buffer: &mut S::Buffer,
        query: &[u8],
    ) -> AlignmentResult {
        if let Some(required_query_length) = self.query_length_checker.optional_length_to_be_allocated(query.len() as u32) {
            self.mode.allocate_space(
                required_query_length,
                &self.regulator,
            );
        }
        
        let reference_alignment_result = self.mode.run_algorithm(
            reference,
            sequence_buffer,
            query,
            &self.regulator,
        );

        self.regulator.result_of_uncompressed_penalty(reference_alignment_result)
    }
}

mod debug;
mod alignments;
pub use alignments::AlignmentError;