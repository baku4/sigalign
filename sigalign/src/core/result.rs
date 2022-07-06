// Result of alignment
use crate::{Deserialize, Serialize};
use serde::ser::{Serializer, SerializeStruct};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
pub struct AlignmentResult(
    pub Vec<RecordAlignmentResult>
);

#[derive(Debug, PartialEq, Eq, Hash, Clone, Deserialize)]
pub struct RecordAlignmentResult {
    pub index: usize,
    pub alignments: Vec<AnchorAlignmentResult>,
}
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
pub struct AnchorAlignmentResult {
    pub penalty: usize,
    pub length: usize,
    pub position: AlignmentPosition,
    pub operations: Vec<AlignmentOperation>,
}
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
pub struct AlignmentPosition {
    pub record: (usize, usize),
    pub query: (usize, usize),
}
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
pub struct AlignmentOperation {
    pub case: AlignmentCase,
    pub count: u32,
}
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
pub enum AlignmentCase {
    Match,
    Subst,
    Insertion,
    Deletion,
}
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
