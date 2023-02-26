use super::AlignmentResult; 
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
pub struct FastaAlignmentResult(
    pub Vec<ReadAlignmentResult>
);

#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "short_key", serde(rename = "ReadAln"))]
pub struct ReadAlignmentResult {
    #[cfg_attr(feature = "short_key", serde(rename = "id"))]
    pub read: String,
    #[cfg_attr(feature = "short_key", serde(rename = "res"))]
    pub result: AlignmentResult,
}
