/*!
Alignment results.
*/ 
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlignmentResult(
    pub Vec<TargetAlignmentResult>
);

#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "short_key", serde(rename = "TgtAln"))]
pub struct TargetAlignmentResult {
    #[cfg_attr(feature = "short_key", serde(rename = "idx"))]
    pub index: u32,
    #[cfg_attr(feature = "short_key", serde(rename = "aln"))]
    pub alignments: Vec<AnchorAlignmentResult>,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "short_key", serde(rename = "AncAln"))]
pub struct AnchorAlignmentResult {
    #[cfg_attr(feature = "short_key", serde(rename = "pen"))]
    pub penalty: u32,
    #[cfg_attr(feature = "short_key", serde(rename = "len"))]
    pub length: u32,
    #[cfg_attr(feature = "short_key", serde(rename = "pos"))]
    pub position: AlignmentPosition,
    #[cfg_attr(feature = "short_key", serde(rename = "ops"))]
    pub operations: Vec<AlignmentOperations>,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
#[derive(Serialize, Deserialize)]
pub struct AlignmentPosition {
    #[cfg_attr(feature = "short_key", serde(rename = "qry"))]
    pub query: (u32, u32),
    #[cfg_attr(feature = "short_key", serde(rename = "tgt"))]
    pub target: (u32, u32),
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "short_key", serde(rename = "Operation"))]
pub struct AlignmentOperations {
    #[cfg_attr(feature = "short_key", serde(rename = "op"))]
    pub operation: AlignmentOperation,
    #[cfg_attr(feature = "short_key", serde(rename = "n"))]
    pub count: u32,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
#[derive(Serialize, Deserialize)]
pub enum AlignmentOperation {
    #[cfg_attr(feature = "short_key", serde(rename = "M"))]
    Match,
    #[cfg_attr(feature = "short_key", serde(rename = "S"))]
    Subst,
    #[cfg_attr(feature = "short_key", serde(rename = "I"))]
    Insertion,
    #[cfg_attr(feature = "short_key", serde(rename = "D"))]
    Deletion,
}

pub mod labeled;
pub mod fasta;
mod to_json;

// Features
mod count_results;
mod deduplicate;
