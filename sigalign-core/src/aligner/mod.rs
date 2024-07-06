/*!
Alignment executors.

- `Regulator` is definition for the alignment results, guiding the alignment process.
- Aligners are built from `Regulator`.

Usage:
```rust
use sigalign_core::aligner::AlignmentRegulator;
use sigalign_core::aligner::local::LocalAligner;

let regulator = AlignmentRegulator::new(
    4,   // Mismatch penalty
    6,   // Gap open penalty
    2,   // Gap extend penalty
    50,  // Minimum length
    0.1, // Maximum penalty per length
).unwrap(); // Can occur `RegulatorError` when input is invalid

let aligner = LocalAligner::new(regulator); // Never fails
```
*/

// Common components
//  - To make workspace
mod workspace;
//  - To define input parameters
mod regulator;
pub use regulator::{AlignmentRegulator, RegulatorError};

/// Executing "local" alignment algorithm.
pub mod local;
// Executing "semi-global" alignment algorithm.
pub mod semi_global;
