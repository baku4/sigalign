// Fasta alignment result
use crate::{Result};
use crate::{Serialize, Deserialize};
use super::{
    AlignmentResult,
};
#[cfg(feature = "short_key")]
use serde::ser::{Serializer, SerializeStruct};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
pub struct FastaAlignmentResult(
    pub Vec<ReadAlignmentResult>
);

#[derive(Debug, PartialEq, Eq, Hash, Clone, Deserialize)]
#[cfg_attr(not(feature = "short_key"), derive(Serialize))]
pub struct ReadAlignmentResult {
    pub read: String,
    pub result: AlignmentResult,
}
#[cfg(feature = "short_key")]
impl Serialize for ReadAlignmentResult {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where
        S: Serializer,
    {
        let mut alignment = serializer.serialize_struct("RdAln", 2)?;
        alignment.serialize_field("id", &self.read)?;
        alignment.serialize_field("res", &self.result)?;
        alignment.end()
    }
}
