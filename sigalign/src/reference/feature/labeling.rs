use super::{
    Result, error_msg,
	Penalties, PRECISION_SCALE, Cutoff, MinPenaltyForPattern,
	AlignmentResult, RecordAlignmentResult, AnchorAlignmentResult, AlignmentPosition, AlignmentOperation, AlignmentCase,
    Sequence,
    ReferenceInterface, PatternLocation,
};
use super::{
    Reference, SequenceStorage,
    SequenceType, PatternFinder, Serializable,
};

impl<SL> Reference<SL> where
    SL: SequenceStorage + LabelStorage,
{
    pub fn label_of_record(&self, record_index: usize) -> String {
        self.sequence_storage.label_of_record(record_index)
    }
}

/// Storage for label of sequences.
pub trait LabelStorage {
    fn label_of_record(&self, record_index: usize) -> String;
}
