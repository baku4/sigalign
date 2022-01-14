// Result of alignment
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ReferenceAlignmentResult(
    pub Vec<RecordAlignmentResult>
);

#[derive(Debug, Serialize, Deserialize)]
pub struct RecordAlignmentResult {
    pub record_index: usize,
    pub result: Vec<AlignmentResult>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AlignmentResult {
    pub penalty: usize,
    pub length: usize,
    pub position: AlignmentPosition,
    pub operations: Vec<AlignmentOperation>,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
pub struct AlignmentPosition {
    pub record: (usize, usize),
    pub query: (usize, usize),
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
pub struct AlignmentOperation {
    pub alignment_type: AlignmentType,
    pub count: u32,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
pub enum AlignmentType {
    Match,
    Subst,
    Insertion,
    Deletion,
}
