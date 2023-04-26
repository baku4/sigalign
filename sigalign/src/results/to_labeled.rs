use super::{
    Reference,
    SequenceStorage,
    LabelStorage,
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
};

impl FastaAlignmentResult {
    pub fn to_labeled<SL>(self, reference: &Reference<SL>) -> FastaAlignmentLabeledResult where
        SL: SequenceStorage + LabelStorage,
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
        SL: SequenceStorage + LabelStorage,
    {
        ReadAlignmentLabeledResult {
            read: self.read,
            result: self.result.to_labeled(reference),
        }
    }
}

impl AlignmentResult {
    pub fn to_labeled<SL>(self, reference: &Reference<SL>) -> AlignmentLabeledResult where
        SL: SequenceStorage + LabelStorage,
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
        SL: SequenceStorage + LabelStorage,
    {
        RecordAlignmentLabeledResult {
            index: self.index,
            label: reference.label_of_record(self.index),
            alignments: self.alignments,
        }
    }
}