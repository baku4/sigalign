use serde::{Deserialize, Serialize};

use super::Alignment;


#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "short_key", serde(rename = "LblQryAln"))]
pub struct LabeledQueryAlignment(
    pub Vec<LabeledTargetAlignment>
);

#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "short_key", serde(rename = "LblTgtAln"))]
pub struct LabeledTargetAlignment {
    #[cfg_attr(feature = "short_key", serde(rename = "idx"))]
    pub index: u32,
    #[cfg_attr(feature = "short_key", serde(rename = "lbl"))]
    pub label: String,
    #[cfg_attr(feature = "short_key", serde(rename = "aln"))]
    pub alignments: Vec<Alignment>,
}
