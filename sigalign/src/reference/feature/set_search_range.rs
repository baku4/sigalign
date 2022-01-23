use super::{
    Result, error_msg,
	Penalties, PRECISION_SCALE, Cutoff, MinPenaltyForPattern,
	AlignmentResult, RecordAlignmentResult, AnchorAlignmentResult, AlignmentPosition, AlignmentOperation, AlignmentCase,
    Sequence,
    ReferenceInterface, PatternLocation,
};
use super::{
    Reference, SequenceProvider,
    SequenceType, PatternFinder,
};

impl<'a, S> Reference<'a, S> where
    S: SequenceProvider<'a>,
{
    pub fn set_search_range(&mut self, mut target_record_index: Vec<u32>) -> Result<()> {
        target_record_index.sort();
        let last_record_index = match target_record_index.last() {
            Some(v) => v,
            None => error_msg!("Record index cannot be empty")
        };
        if (self.sequence_provider.total_record_count() as u32) < *last_record_index {
            error_msg!("Record index is out of bound")
        } else {
            self.set_search_range_unchecked(target_record_index);
            Ok(())
        }
    }
    pub fn set_search_range_unchecked(&mut self, sorted_inbound_target_record_index: Vec<u32>) {
        self.target_record_index = sorted_inbound_target_record_index;
    }
}