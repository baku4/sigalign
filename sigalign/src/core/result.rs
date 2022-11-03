// Result of alignment
use crate::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlignmentResult(
    pub Vec<RecordAlignmentResult>
);

#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "short_key", serde(rename = "RecAln"))]
pub struct RecordAlignmentResult {
    #[cfg_attr(feature = "short_key", serde(rename = "idx"))]
    pub index: usize,
    #[cfg_attr(feature = "short_key", serde(rename = "aln"))]
    pub alignments: Vec<AnchorAlignmentResult>,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "short_key", serde(rename = "AncAln"))]
pub struct AnchorAlignmentResult {
    #[cfg_attr(feature = "short_key", serde(rename = "pen"))]
    pub penalty: usize,
    #[cfg_attr(feature = "short_key", serde(rename = "len"))]
    pub length: usize,
    #[cfg_attr(feature = "short_key", serde(rename = "pos"))]
    pub position: AlignmentPosition,
    #[cfg_attr(feature = "short_key", serde(rename = "ops"))]
    pub operations: Vec<AlignmentOperation>,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
#[derive(Serialize, Deserialize)]
pub struct AlignmentPosition {
    #[cfg_attr(feature = "short_key", serde(rename = "rec"))]
    pub record: (usize, usize),
    #[cfg_attr(feature = "short_key", serde(rename = "qry"))]
    pub query: (usize, usize),
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "short_key", serde(rename = "Operation"))]
pub struct AlignmentOperation {
    #[cfg_attr(feature = "short_key", serde(rename = "op"))]
    pub case: AlignmentCase,
    #[cfg_attr(feature = "short_key", serde(rename = "n"))]
    pub count: u32,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
#[derive(Serialize, Deserialize)]
pub enum AlignmentCase {
    #[cfg_attr(feature = "short_key", serde(rename = "M"))]
    Match,
    #[cfg_attr(feature = "short_key", serde(rename = "S"))]
    Subst,
    #[cfg_attr(feature = "short_key", serde(rename = "I"))]
    Insertion,
    #[cfg_attr(feature = "short_key", serde(rename = "D"))]
    Deletion,
}
