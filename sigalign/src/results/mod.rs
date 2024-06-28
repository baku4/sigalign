/*!
The structures for representing alignment outcomes.

- `LabeledTargetAlignment`: Contains a target index and label, with alignments against a specific target. 
    ```rust
    index: u32,
    label: String,
    alignments: Vec<AnchorAlignmentResult>,
    ```

- `Alignment`: Details alignment's penalty, length, position, and operations.
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
