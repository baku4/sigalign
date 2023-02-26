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

mod algorithms;
use algorithms::{WaveFront, local_alignment_algorithm, semi_global_alignment_algorithm};

mod wave_front_pool;
use wave_front_pool::{
    WaveFrontPool, SingleWaveFrontPool, DoubleWaveFrontPool,
};
pub use wave_front_pool::{
    AllocationStrategy, LinearStrategy, DoublingStrategy,
};

mod regulator;
use regulator::{AlignmentRegulator};
pub use regulator::{RegulatorError};

// Modes
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
}

// Features
// mod feature;
