use super::{
    Result, error_msg,
	Penalties, PRECISION_SCALE, Cutoff, MinPenaltyForPattern,
	AlignmentResult, RecordAlignmentResult, AnchorAlignmentResult, AlignmentPosition, AlignmentOperation, AlignmentCase,
    Sequence,
    ReferenceInterface, PatternLocation,
    Reference, SequenceProvider,
    AlignmentCondition,
    SemiGlobalAligner, LocalAligner,
    Aligner, Algorithms, AlignerInterface,
};

impl Aligner {
    pub fn get_penalties(&self) -> [usize; 3] {
        self.get_condition().get_penalties()
    }
    pub fn get_similarity_cutoff(&self) -> (usize, f32) {
        self.get_condition().get_similarity_cutoff()
    }
    pub fn get_pattern_size(&self) -> usize {
        self.get_condition().pattern_size
    }
    fn get_condition(&self) -> &AlignmentCondition {
        match &self.algorithms {
            Algorithms::SemiGlobal(aligner) => &aligner.condition,
            Algorithms::Local(aligner) => &aligner.condition,
        }
    }
}