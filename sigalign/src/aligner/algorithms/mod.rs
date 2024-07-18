/*!
Supported alignment algorithms.

`Algorithm` defines the behavior for an `Aligner`. It needs to be passed to build an `Aligner`.
e.g.,
```rust
use sigalign::{Aligner, algorithms::Local};

let algorithm = Local::new(4, 6, 2, 50, 0.1).unwrap();
let aligner = Aligner::new(algorithm);
```

The `Algorithm` trait is not intended to be used directly. Instead, use the implemented algorithms.
Currently, the following algorithms are implemented:

1. **Basic**: Basic algorithm without constraints.
   - `Local`: Performs local alignment.
   - `SemiGlobal`: Performs semi-global alignment.

2. **With Limit**: Performs alignment with a limit on the number of alignments. 
   The algorithm stops after finding a certain number of alignments that satisfy the cutoffs, 
   which means that the results are not guaranteed to be optimal.
   - `LocalWithLimit`: Local alignment with a limit.
   - `SemiGlobalWithLimit`: Semi-global alignment with a limit.

3. **With Chunk**: Divides the query sequence into chunks and aligns each chunk separately.
   This is useful when the query sequence is too long to be aligned at once.
   - `LocalWithChunk`: Local alignment with chunking.
   - `SemiGlobalWithChunk`: Semi-global alignment with chunking.

## Local vs SemiGlobal

The alignment mode in bioinformatics dictates how sequences are compared and aligned. SigAlign supports two modes: semi-global and local.

In the **semi-global** mode, either the query or the reference sequence is completely consumed at each alignment end. Examples include:
- Case 1
    ```text
    QUERY : -------------
                |||||||||
    TARGET:     -------------
    ```
- Case 2
    ```text
    QUERY :     -------------
                |||||||||
    TARGET: -------------
    ```
- Case 3
    ```text
    QUERY : -------------
              |||||||
    TARGET:   -------
    ```
- Case 4
    ```text
    QUERY :   -------
              |||||||
    TARGET: -------------
    ```

In the **local** mode, the alignment may include only parts of the target and query sequence.
For example:
- Case 1
    ```text
    QUERY : ----------------
                  ||||||
    TARGET:    ----------------
    ```
 */

use sigalign_core::aligner::AlignmentRegulator;
use super::{
    Reference, DefaultSequenceBuffer,
    QueryAlignment,
};

mod error;
pub use error::ParamsError;
use error::check_pattern_size;

mod basic;
mod with_limit;
mod with_chunk;
pub use basic::{Local, SemiGlobal};
pub use with_limit::{LocalWithLimit, SemiGlobalWithLimit};
pub use with_chunk::{LocalWithChunk, SemiGlobalWithChunk};

/// An alignment algorithm.
pub trait Algorithm: std::fmt::Debug + Clone {
    // Low-level alignment method
    fn align(
        &mut self,
        query: &[u8],
        reference: &Reference,
        sequence_buffer: &mut DefaultSequenceBuffer,
    ) -> QueryAlignment;
    // Can access the regulator
    fn regulator(&self) -> &AlignmentRegulator;
}
