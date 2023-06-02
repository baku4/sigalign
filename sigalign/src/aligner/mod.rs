/*!
An alignment executor that performs sequence alignment.
# Aligner
The `Aligner` receives two values: (1) the query sequence, and (2) the reference struct, and returns the alignment result.

The `Aligner` is responsible for managing the workspace required for alignment:
  - It has a cache for alignment extension.
    - The size of the cache is proportional to the square of the query length.
    - The semi-global mode uses about half of the cache compared to the local mode for the same query length.
  - It automatically controls the sequence buffer when a reference is passed.
## Mode
The `Aligner` has two modes:
  1. Semi-global algorithm
     - At each end of the alignment, either the query or the reference is fully consumed.
       For example:
         * Case 1
           ```text
           QUERY : -------------
                       |||||||||
           TARGET:     -------------
           ```
         * Case 2
           ```text
           QUERY :     -------------
                       |||||||||
           TARGET: -------------
           ```
         * Case 3
           ```text
           QUERY : -------------
                      |||||||
           TARGET:    -------
           ```
         * Case 4
           ```text
           QUERY :    -------
                      |||||||
           TARGET: -------------
           ```
  2. Local algorithm
     - The alignment can include only parts of the target and query sequence.
       For example:
         ```text
         QUERY : ----------------
                     |||||||
         TARGET:    ----------------
         ```
     - The result is the same as the semi-global alignments of all substrings in the query sequence.
*/
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