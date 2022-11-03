// Labeled alignment result
use crate::{Serialize, Deserialize};
use super::{
    AnchorAlignmentResult,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FastaAlignmentLabeledResult(
    pub Vec<ReadAlignmentLabeledResult>
);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadAlignmentLabeledResult {
    pub read: String,
    pub result: AlignmentLabeledResult,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlignmentLabeledResult(
    pub Vec<RecordAlignmentLabeledResult>
);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordAlignmentLabeledResult {
    pub index: usize,
    pub label: String,
    pub alignments: Vec<AnchorAlignmentResult>,
}
