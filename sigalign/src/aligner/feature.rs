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

mod debug;
mod cmp;
mod clean_cache;

impl Aligner {
    pub fn alignment_unchecked<S: SequenceProvider>(
        &mut self,
        reference: &Reference<S>,
        query: Sequence,
    ) -> AlignmentResult {
        match &mut self.algorithms {
            Algorithms::SemiGlobal(aligner) => aligner.alignment(reference, query),
            Algorithms::Local(aligner) => aligner.alignment(reference, query),
        }
    }
    pub fn alignment_checked<S: SequenceProvider>(
        &mut self,
        reference: &Reference<S>,
        query: Sequence,
    ) -> Result<AlignmentResult> {
        if !reference.searchable(query) {
            error_msg!("Query contains unsearchable character")
        }
        Ok(match &mut self.algorithms {
            Algorithms::SemiGlobal(aligner) => aligner.alignment(reference, query),
            Algorithms::Local(aligner) => aligner.alignment(reference, query),
        })
    }
}
