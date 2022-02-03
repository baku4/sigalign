// Result of alignment
use crate::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
pub struct AlignmentResult(
    pub Vec<RecordAlignmentResult>
);

#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
pub struct RecordAlignmentResult {
    pub index: usize,
    pub alignments: Vec<AnchorAlignmentResult>,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
pub struct AnchorAlignmentResult {
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
    pub case: AlignmentCase,
    pub count: u32,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
pub enum AlignmentCase {
    Match,
    Subst,
    Insertion,
    Deletion,
}
