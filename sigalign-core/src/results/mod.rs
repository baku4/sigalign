/*!
Alignment results.

Example in JSON format:
```json
{
  "QueryAlignment": [
    {
      "TargetAlignment": {
        "index": 0,
        "alignments": [
          {
            "Alignment": {
              "penalty": 4,
              "length": 100,
              "position": {
                "query": [0, 100],
                "target": [0, 100]
              },
              "operations": [
                {
                  "AlignmentOperations": {
                    "operation": "Match",
                    "count": 99
                  }
                },
                {
                  "AlignmentOperations": {
                    "operation": "Subst",
                    "count": 1
                  }
                }
              ]
            }
          }
        ]
      }
    }
  ]
}
```
*/ 
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "short_key", serde(rename = "QryAln"))]
pub struct QueryAlignment(
    pub Vec<TargetAlignment>
);

#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "short_key", serde(rename = "TgtAln"))]
pub struct TargetAlignment {
    #[cfg_attr(feature = "short_key", serde(rename = "idx"))]
    pub index: u32,
    #[cfg_attr(feature = "short_key", serde(rename = "aln"))]
    pub alignments: Vec<Alignment>,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "short_key", serde(rename = "Aln"))]
pub struct Alignment {
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
#[cfg_attr(feature = "short_key", serde(rename = "Pos"))]
pub struct AlignmentPosition {
    #[cfg_attr(feature = "short_key", serde(rename = "qry"))]
    pub query: (u32, u32),
    #[cfg_attr(feature = "short_key", serde(rename = "tgt"))]
    pub target: (u32, u32),
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "short_key", serde(rename = "Ops"))]
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
    #[cfg_attr(feature = "short_key", serde(rename = "D"))]
    Deletion,
    #[cfg_attr(feature = "short_key", serde(rename = "I"))]
    Insertion,
}

mod to_json;

// Features
mod count_alignments;
mod deduplicate;
