use super::{
    Result, error_msg,
	Penalties, PRECISION_SCALE, Cutoff, MinPenaltyForPattern,
	AlignmentResult, RecordAlignmentResult, AnchorAlignmentResult, AlignmentPosition, AlignmentOperation, AlignmentCase,
    Sequence,
    ReferenceInterface, PatternLocation,
    Reference, SequenceStorage,
    AlignmentCondition,
    SemiGlobalAligner, LocalAligner,
    Aligner, Algorithms, AlignerInterface,
};

/// Print status of [Aligner]
impl Aligner {
    /// Print penalties: (1) mismatch penalty, (2) gap-open penalty, and (3) gap-extend penalty
    pub fn get_penalties(&self) -> [usize; 3] {
        self.get_condition().get_penalties()
    }
    /// Print cut-offs: (1) minimum length and (2) maximum penalty per length
    pub fn get_similarity_cutoff(&self) -> (usize, f32) {
        self.get_condition().get_similarity_cutoff()
    }
    /// Print pattern size
    pub fn get_pattern_size(&self) -> usize {
        self.get_condition().pattern_size
    }
    /// Print type of algorithm
    pub fn is_local_mode(&self) -> bool {
        match &self.algorithms {
            Algorithms::SemiGlobal(_) => false,
            Algorithms::Local(_) => true,
        }
    }
    fn get_condition(&self) -> &AlignmentCondition {
        match &self.algorithms {
            Algorithms::SemiGlobal(aligner) => &aligner.condition,
            Algorithms::Local(aligner) => &aligner.condition,
        }
    }
}