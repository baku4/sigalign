// Fasta alignment result
use crate::{Result, error_msg};
use crate::{Serialize, Deserialize};
use serde::ser::{Serializer, SerializeStruct};
use super::{
    AlignmentResult,
    RecordAlignmentResult,
    AnchorAlignmentResult,
    AlignmentPosition,
    AlignmentOperation,
    AlignmentCase,
};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
pub struct FastaAlignmentResult(
    pub Vec<ReadAlignmentResult>
);

#[derive(Debug, PartialEq, Eq, Hash, Clone, Deserialize)]
pub struct ReadAlignmentResult {
    pub read: String,
    pub result: AlignmentResult,
}
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
