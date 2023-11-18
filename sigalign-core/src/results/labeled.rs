use serde::{Deserialize, Serialize};
use crate::reference::{
    Reference, PatternIndex, SequenceStorage,
    extensions::LabelStorage,
};
use super::{
    AlignmentResult,
    AnchorAlignmentResult,
    TargetAlignmentResult,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabeledAlignmentResult(
    pub Vec<LabeledTargetAlignmentResult>
);

#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "short_key", serde(rename = "TgtAln"))]
pub struct LabeledTargetAlignmentResult {
    #[cfg_attr(feature = "short_key", serde(rename = "idx"))]
    pub index: u32,
    #[cfg_attr(feature = "short_key", serde(rename = "lbl"))]
    pub label: String,
    #[cfg_attr(feature = "short_key", serde(rename = "aln"))]
    pub alignments: Vec<AnchorAlignmentResult>,
}

impl AlignmentResult {
    pub fn to_labeled_result_unchecked<I, S>(
        self,
        reference: &Reference<I, S>,
    ) -> LabeledAlignmentResult where
        I: PatternIndex,
        S: SequenceStorage + LabelStorage,
    {
        LabeledAlignmentResult(self.0.into_iter().map(|x| {
            x.to_labeled_result_unchecked(reference)
        }).collect())
    }
}
impl TargetAlignmentResult {
    fn to_labeled_result_unchecked<I, S>(
        self,
        reference: &Reference<I, S>,
    ) -> LabeledTargetAlignmentResult where
        I: PatternIndex,
        S: SequenceStorage + LabelStorage,
    {
        LabeledTargetAlignmentResult {
            index: self.index,
            label: reference.label_of_target_unchecked(self.index),
            alignments: self.alignments
        }
    }
}
