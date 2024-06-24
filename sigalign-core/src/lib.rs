/*!
The collection of the most **fundamental components** to execute the SigAlign algorithm.

The purpose of `sigalign-core` is to hide the complexity of the detailed implementation of the SigAlign algorithm.
- `sigalign-core` allows for lower-level control compared to `sigalign` crate.
- `sigalign-core` can be used to perform more optimized alignment for specific tasks.
    - Implementations for traits such as `SequenceStorage` and `PatternIndex` are required (manually or by using `sigalign-impl`).
- `sigalign-core` does not have user-friendly interfaces implemented in `sigalign`.
*/

mod core;
mod algorithm;

pub mod results;
pub mod reference;
pub mod aligner;
