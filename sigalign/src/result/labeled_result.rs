// Labeled alignment result
use crate::{Result, error_msg};
use crate::{Serialize, Deserialize};
use super::{
    AlignmentResult,
    RecordAlignmentResult,
    AnchorAlignmentResult,
    AlignmentPosition,
    AlignmentOperation,
    AlignmentCase,
};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
pub struct FastaAlignmentLabeledResult(
    pub Vec<ReadAlignmentLabeledResult>
);

#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
pub struct ReadAlignmentLabeledResult {
    pub read: String,
    pub alignment: AlignmentLabeledResult,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
pub struct AlignmentLabeledResult(
    pub Vec<RecordAlignmentLabeledResult>
);

#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
pub struct RecordAlignmentLabeledResult {
    pub index: usize,
    pub label: String,
    pub result: Vec<AnchorAlignmentResult>,
}
