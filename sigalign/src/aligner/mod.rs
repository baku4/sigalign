use crate::core::{
    regulators::{
        Penalty, PREC_SCALE, Cutoff, MinPenaltyForPattern,
    },
    results::{
        AlignmentResult, RecordAlignmentResult, AnchorAlignmentResult, AlignmentPosition, AlignmentOperation, AlignmentCase,
    },
    ReferenceInterface, SequenceBuffer, PatternLocation,
};
pub use crate::reference::{
    Reference,
    pattern_index::PatternIndex,
    sequence_storage::SequenceStorage,
};

// Core algorithms
mod algorithm;
use algorithm::{WaveFront, local_alignment_algorithm, semi_global_alignment_algorithm};

// Common data structures for aligner
//  - Cache for alignment extension
mod wave_front_cache;
use wave_front_cache::{WaveFrontCache, SingleWaveFrontCache, DoubleWaveFrontCache};
//  - Alignment condition
mod alignment_condition;
pub use alignment_condition::{
    AlignmentCondition,
    calculate_max_pattern_size,
};

// Aligner interface
pub trait AlignerInterface {
    fn new(condition: AlignmentCondition) -> Self where Self: Sized;
    fn alignment<S>(
        &mut self,
        reference: &Reference<S>,
        sequence_buffer: &mut S::Buffer,
        query: Sequence,
    ) -> AlignmentResult where S: SequenceStorage;
}

// Aligner implementations
mod semi_global;
mod local;
pub use semi_global::SemiGlobalAligner;
pub use local::LocalAligner;

// Features
mod feature;

/**
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
#[derive(Clone)]
pub struct Aligner {
    pub(crate) algorithms: Algorithms,
}

#[derive(Clone)]
pub enum Algorithms {
    SemiGlobal(SemiGlobalAligner),
    Local(LocalAligner),
}

/// Basic methods
impl Aligner {
    /// Create [Aligner] with semi-global mode
    pub fn new_semi_global(
        mismatch_penalty: usize,
        gap_open_penalty: usize,
        gap_extend_penalty: usize,
        minimum_aligned_length: usize,
        maximum_penalty_per_length: f32,
    ) -> Result<Self> {
        let alignment_condition = AlignmentCondition::new(mismatch_penalty, gap_open_penalty, gap_extend_penalty, minimum_aligned_length, maximum_penalty_per_length)?;
        Ok(Self {
            algorithms: Algorithms::SemiGlobal(SemiGlobalAligner::new(alignment_condition))
        })
    }
    /// Create [Aligner] with local mode
    pub fn new_local(
        mismatch_penalty: usize,
        gap_open_penalty: usize,
        gap_extend_penalty: usize,
        minimum_aligned_length: usize,
        maximum_penalty_per_length: f32,
    ) -> Result<Self> {
        let alignment_condition = AlignmentCondition::new(mismatch_penalty, gap_open_penalty, gap_extend_penalty, minimum_aligned_length, maximum_penalty_per_length)?;
        Ok(Self {
            algorithms: Algorithms::Local(LocalAligner::new(alignment_condition))
        })
    }
    /// Perform alignment with reference and sequence buffer
    pub fn alignment<S: SequenceStorage>(
        &mut self,
        reference: &Reference<S>,
        sequence_buffer: &mut S::Buffer,
        query: Sequence,
    ) -> AlignmentResult {
        match &mut self.algorithms {
            Algorithms::SemiGlobal(aligner) => aligner.alignment(reference, sequence_buffer, query),
            Algorithms::Local(aligner) => aligner.alignment(reference, sequence_buffer, query),
        }
    }
}
