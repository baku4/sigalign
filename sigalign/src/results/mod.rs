/*!
Alignment results.
*/ 
use serde::{Deserialize, Serialize};

// Re-export sigalign-core results
pub use sigalign_core::results::{
    AlignmentResult,
    TargetAlignmentResult,
    AnchorAlignmentResult,
    AlignmentPosition,
    AlignmentOperations,
    AlignmentOperation,
    labeled::{
        LabeledAlignmentResult,
        LabeledTargetAlignmentResult,
    },
};

#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "short_key", serde(rename = "ReadAln"))]
pub struct ReadAlignmentResult {
    #[cfg_attr(feature = "short_key", serde(rename = "id"))]
    pub read: String,
    #[cfg_attr(feature = "short_key", serde(rename = "+"))]
    pub is_forward: bool,
    #[cfg_attr(feature = "short_key", serde(rename = "res"))]
    pub result: LabeledAlignmentResult,
}

#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
pub struct FastaAlignmentResult(
    pub Vec<ReadAlignmentResult>
);

// pub mod fasta;
// pub mod to_sam;
