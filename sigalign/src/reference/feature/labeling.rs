use super::{
    Result, error_msg,
	Penalties, PRECISION_SCALE, Cutoff, MinPenaltyForPattern,
	AlignmentResult, RecordAlignmentResult, AnchorAlignmentResult, AlignmentPosition, AlignmentOperation, AlignmentCase,
    Sequence,
    ReferenceInterface, PatternLocation,
};
use super::{
    Reference, SequenceProvider,
    SequenceType, PatternFinder, Serializable,
};

impl<SL> Reference<SL> where
    SL: SequenceProvider + LabelProvider,
{
    pub fn label_of_record(&self, record_index: usize) -> String {
        self.sequence_provider.label_of_record(record_index)
    }
}

pub trait LabelProvider {
    fn label_of_record(&self, record_index: usize) -> String;
}
