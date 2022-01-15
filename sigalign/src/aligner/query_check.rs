use super::{Result, error_msg};
use super::{
	Penalties, PRECISION_SCALE, Cutoff, MinPenaltyForPattern,
	ReferenceAlignmentResult, RecordAlignmentResult, AlignmentResult, AlignmentPosition, AlignmentOperation, AlignmentType,
    Sequence,
    ReferenceInterface, PatternLocation,
    AlignerInterface,
};

pub trait SearchableCheck: AlignerInterface {
    fn alignment_checked(
        &mut self,
        reference: &mut dyn ReferenceInterface,
        query: Sequence,
    ) -> Result<ReferenceAlignmentResult> {
        if reference.searchable(query) {
            Ok(self.alignment(reference, query))
        } else {
            error_msg!("Unsearchable query")
        }
    }
}
