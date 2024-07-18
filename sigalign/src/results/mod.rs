/*!
Representation of the alignment outcomes.

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
*/
mod labeled;

// Re-export sigalign-core results
pub use sigalign_core::results::{
    QueryAlignment,
    TargetAlignment,
    Alignment,
    AlignmentPosition,
    AlignmentOperations,
    AlignmentOperation,
};
// Export labeled results
pub use labeled::{
    LabeledQueryAlignment,
    LabeledTargetAlignment,
};

mod to_json;
mod count_alignments;
