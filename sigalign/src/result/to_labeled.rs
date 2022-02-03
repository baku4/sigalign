use crate::{Result, error_msg};
use crate::core::{
	Penalties, PRECISION_SCALE, Cutoff, MinPenaltyForPattern,
    Sequence,
    ReferenceInterface, SequenceBuffer, PatternLocation,
};
use super::{
    Reference,
    SequenceProvider,
    LabelProvider,
};
use super::{
    FastaAlignmentLabeledResult,
    ReadAlignmentLabeledResult,
    AlignmentLabeledResult,
    RecordAlignmentLabeledResult,
    FastaAlignmentResult,
    ReadAlignmentResult,
    AlignmentResult,
    RecordAlignmentResult,
    AnchorAlignmentResult,
    AlignmentPosition,
    AlignmentOperation,
    AlignmentCase,
};

impl FastaAlignmentResult {
    pub fn to_labeled<SL>(self, reference: &Reference<SL>) -> FastaAlignmentLabeledResult where
        SL: SequenceProvider + LabelProvider,
    {
        FastaAlignmentLabeledResult(
            self.0.into_iter().map(|read_alignment_result| {
                read_alignment_result.to_labeled(reference)
            }).collect()
        )
    }
}

impl ReadAlignmentResult {
    pub fn to_labeled<SL>(self, reference: &Reference<SL>) -> ReadAlignmentLabeledResult where
        SL: SequenceProvider + LabelProvider,
    {
        ReadAlignmentLabeledResult {
            read: self.read,
            alignment: self.alignment.to_labeled(reference),
        }
    }
}

impl AlignmentResult {
    pub fn to_labeled<SL>(self, reference: &Reference<SL>) -> AlignmentLabeledResult where
        SL: SequenceProvider + LabelProvider,
    {
        AlignmentLabeledResult(
            self.0.into_iter().map(|record_alignment_result| {
                record_alignment_result.to_labeled(reference)
            }).collect()
        )
    }
}

impl RecordAlignmentResult {
    pub fn to_labeled<SL>(self, reference: &Reference<SL>) -> RecordAlignmentLabeledResult where
        SL: SequenceProvider + LabelProvider,
    {
        RecordAlignmentLabeledResult {
            index: self.index,
            label: reference.label_of_record(self.index),
            result: self.result,
        }
    }
}