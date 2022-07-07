// Result of alignment
use crate::{Deserialize, Serialize};
#[cfg(feature = "short_key")]
use serde::ser::{Serializer, SerializeStruct};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
pub struct AlignmentResult(
    pub Vec<RecordAlignmentResult>
);

#[derive(Debug, PartialEq, Eq, Hash, Clone, Deserialize)]
#[cfg_attr(not(feature = "short_key"), derive(Serialize))]

pub struct RecordAlignmentResult {
    pub index: usize,
    pub alignments: Vec<AnchorAlignmentResult>,
}
#[cfg(feature = "short_key")]
impl Serialize for RecordAlignmentResult {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where
        S: Serializer,
    {
        let mut alignment = serializer.serialize_struct("RecAln", 2)?;
        alignment.serialize_field("idx", &self.index)?;
        alignment.serialize_field("aln", &self.alignments)?;
        alignment.end()
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Deserialize)]
#[cfg_attr(not(feature = "short_key"), derive(Serialize))]
pub struct AnchorAlignmentResult {
    pub penalty: usize,
    pub length: usize,
    pub position: AlignmentPosition,
    pub operations: Vec<AlignmentOperation>,
}
#[cfg(feature = "short_key")]
impl Serialize for AnchorAlignmentResult {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where
        S: Serializer,
    {
        let mut alignment = serializer.serialize_struct("AncAln", 4)?;
        alignment.serialize_field("pen", &self.penalty)?;
        alignment.serialize_field("len", &self.length)?;
        alignment.serialize_field("pos", &self.position)?;
        alignment.serialize_field("ops", &self.operations)?;
        alignment.end()
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Deserialize)]
#[cfg_attr(not(feature = "short_key"), derive(Serialize))]
pub struct AlignmentPosition {
    pub record: (usize, usize),
    pub query: (usize, usize),
}
#[cfg(feature = "short_key")]
impl Serialize for AlignmentPosition {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where
        S: Serializer,
    {
        let mut position = serializer.serialize_struct("Pos", 2)?;
        position.serialize_field("rec", &self.record)?;
        position.serialize_field("qry", &self.query)?;
        position.end()
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Deserialize)]
#[cfg_attr(not(feature = "short_key"), derive(Serialize))]
pub struct AlignmentOperation {
    pub case: AlignmentCase,
    pub count: u32,
}
#[cfg(feature = "short_key")]
impl Serialize for AlignmentOperation {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where
        S: Serializer,
    {
        let mut operation = serializer.serialize_struct("Operation", 2)?;
        operation.serialize_field("op", &self.case)?;
        operation.serialize_field("n", &self.count)?;
        operation.end()
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Deserialize)]
#[cfg_attr(not(feature = "short_key"), derive(Serialize))]
pub enum AlignmentCase {
    Match,
    Subst,
    Insertion,
    Deletion,
}
#[cfg(feature = "short_key")]
impl Serialize for AlignmentCase {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where
        S: Serializer,
    {
        match *self {
            Self::Match => serializer.serialize_unit_variant("Case", 0, "M"),
            Self::Subst => serializer.serialize_unit_variant("Case", 1, "S"),
            Self::Insertion => serializer.serialize_unit_variant("Case", 2, "I"),
            Self::Deletion => serializer.serialize_unit_variant("Case", 3, "D"),
        }
    }
}
