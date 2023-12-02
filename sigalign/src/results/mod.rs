/*!
The structures for representing alignment outcomes.

- `ReadAlignmentResult`: Represents alignment for a single read.
    ```rust
    read: String,
    is_forward: bool,
    result: LabeledAlignmentResult,
    ```

- `FastaAlignmentResult`: Collection of `ReadAlignmentResult` from a FASTA file.
    ```rust
    Vec<ReadAlignmentResult>,
    ```

- `AlignmentResult`: Core structure storing alignment results.
    ```rust
    Vec<TargetAlignmentResult>,
    ```

- `TargetAlignmentResult`: Contains target index and alignments against a target.
    ```rust
    index: u32,
    alignments: Vec<AnchorAlignmentResult>,
    ```

- `AnchorAlignmentResult`: Details alignment's penalty, length, position, and operations.
    ```rust
    penalty: u32,
    length: u32,
    position: AlignmentPosition,
    operations: Vec<AlignmentOperations>,
    ```

- `AlignmentPosition`: Specifies alignment positions in query and target sequences.
    ```rust
    query: (u32, u32),
    target: (u32, u32),
    ```

- `AlignmentOperations`: Describes a sequence of alignment operations.
    ```rust
    operation: AlignmentOperation,
    count: u32,
    ```

- `AlignmentOperation`: Enumerates types of alignment operations.
    ```rust
    Match, Subst, Insertion, Deletion,
    ```
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

mod to_json;
//TODO: pub mod to_sam;

mod count_alignments;
