/*!
Alignment executor

- Aligner has two modes
    1. Semi-global algorithm
        * At each end of the alignment, either the query or the reference is fully consumed.
        * For example,
            * Case 1
                ```text
                QUERY : -------------
                            |||||||||
                RECORD:     -------------
                ```
            * Case 2
                ```text
                QUERY :     -------------
                            |||||||||
                RECORD: -------------
                ```
            * Case 3
                ```text
                QUERY : -------------
                           |||||||
                RECORD:    -------
                ```
            * Case 4
                ```text
                QUERY :    -------
                           |||||||
                RECORD: -------------
                ```
    2. Local algorithm
        * The alignment can contain only parts of record and query sequence.
        * For example,
            ```text
            QUERY : ----------------
                          |||||||
            RECORD:    ----------------
            ```
        * The result is the same as the semi-global alignments of all substrings in the query sequence.

- Aligner controls work space required to alignment.
    - Aligner has cache for alignment extension.
        - The size of the cache is proportional to the square of the query length.
        - The semi-global mode uses about half of the cache than local mode for the same query length.
    - Aligner automatically controls sequence buffer when a reference is passed.
*/
use crate::core::ReferenceInterface;
use crate::results::AlignmentResult;

use super::algorithm::{
    // Algorithms
    semi_global_alignment_algorithm,
    local_alignment_algorithm,
    // Structs to be buffered
    //   - common
    WaveFront,
    //   - local
    AnchorIndex, LocalSparePenaltyCalculator,  LocalExtension, Vpc,
};

// Specifications for the aligners
mod allocation_strategy;
pub use allocation_strategy::{
    AllocationStrategy, LinearStrategy, DoublingStrategy,
};
use allocation_strategy::QueryLengthChecker;
mod wave_front_pool;
use wave_front_pool::{
    WaveFrontPool, SingleWaveFrontPool, DoubleWaveFrontPool,
};
mod regulator;
use regulator::{AlignmentRegulator};
pub use regulator::{RegulatorError};

// Aligners by mode
mod local;
mod semi_global;
pub use local::LocalAligner;
pub use semi_global::SemiGlobalAligner;

pub trait AlignerInterface: Sized {
    fn new(
        mismatch_penalty: u32,
        gap_open_penalty: u32,
        gap_extend_penalty: u32,
        minimum_aligned_length: u32,
        maximum_penalty_per_length: f32,
    ) -> Result<Self, RegulatorError>;
    fn alignment<R: ReferenceInterface>(
        &mut self,
        reference: &R,
        sequence_buffer: &mut R::Buffer,
        query: &[u8],
    ) -> AlignmentResult;

    fn get_mismatch_penalty(&self) -> u32;
    fn get_gap_open_penalty(&self) -> u32;
    fn get_gap_extend_penalty(&self) -> u32;
    fn get_minimum_aligned_length(&self) -> u32;
    fn get_maximum_penalty_per_length(&self) -> f32;
    fn get_pattern_size(&self) -> u32;
}

// Features
mod features;
