use super::{
    Result, error_msg,
	Penalties, PRECISION_SCALE, Cutoff, MinPenaltyForPattern,
	ReferenceAlignmentResult, RecordAlignmentResult, AlignmentResult, AlignmentPosition, AlignmentOperation, AlignmentType,
    Sequence,
    ReferenceInterface, PatternLocation,
    AlignerInterface,
    Aligner,
};

impl Aligner {
    pub fn alignment_unchecked(
        &mut self,
        reference: &dyn ReferenceInterface,
        query: Sequence,
    ) -> ReferenceAlignmentResult {
        match self {
            Self::SemiGlobal(aligner) => aligner.alignment(reference, query),
            Self::Local(aligner) => aligner.alignment(reference, query),
        }
    }
    pub fn alignment_checked(
        &mut self,
        reference: &dyn ReferenceInterface,
        query: Sequence,
    ) -> Result<ReferenceAlignmentResult> {
        if !reference.searchable(query) {
            error_msg!("Query contains unsearchable character")
        }
        Ok(match self {
            Self::SemiGlobal(aligner) => aligner.alignment(reference, query),
            Self::Local(aligner) => aligner.alignment(reference, query),
        })
    }
}
