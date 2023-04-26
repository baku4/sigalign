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


// Reverse complementary

#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
pub struct FastaReverseComplementAlignmentResult(
    pub Vec<ReadReverseComplementAlignmentResult>
);

#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "short_key", serde(rename = "ReadRcAln"))]
pub struct ReadReverseComplementAlignmentResult {
    #[cfg_attr(feature = "short_key", serde(rename = "id"))]
    pub read: String,
    #[cfg_attr(feature = "short_key", serde(rename = "+"))]
    pub is_forward: bool,
    #[cfg_attr(feature = "short_key", serde(rename = "res"))]
    pub result: AlignmentResult,
}
