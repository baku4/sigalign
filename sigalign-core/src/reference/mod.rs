/*!
A database for multiple targeted sequences.

## What is `Reference`?
- `Reference` is a target of alignment, containing multiple sequences.
- It is primarily used by the `Aligner` to perform alignments.

## Features
- `Reference` is designed to be **agnostic** to the storage and retrieval methods for sequences.
    - Sequences could be stored in memory, files, or remote physical locations accessible over a network.
- During alignment, `Reference` remains **immutable**.
    - `Aligner` manages the `SequenceBuffer` defined in `SequenceStorage` to store temporary sequences.
    - `Reference` only defines the how to access the target sequences.
- The **target sequences' range can be adjusted** after building the `Reference`, unlike conventional "Reference" in bioinformatics.
- Basically, `Reference` is simply built from `SequenceStorage` and `PatternIndex::Option` in `sigalign-core`.

## Internal Traits
- [SequenceStorage]: Fetches a target sequence based on a given target index.
- [PatternIndex]: Accepts pattern bytes and returns the indices of the targets exactly matching the pattern. This is necessary for the SigAlign algorithm to exactly locate all patterns in the target sequences.
*/

// Internal components
mod pattern_index;
mod sequence_storage;
// Implementations
mod pattern_locate; // Implements the `BufferedPatternLocater` trait.
mod debug;
// Extensions for additional features for `Reference`.
pub mod extensions;

pub use pattern_index::PatternIndex;
pub use sequence_storage::SequenceStorage;
pub use crate::core::{PatternLocation, SequenceBuffer};

/// A database for multiple target sequences.
#[derive(Debug)]
pub struct Reference<I, S> where
    I: PatternIndex,
    S: SequenceStorage,
{
    target_boundaries: Vec<u32>,
    pattern_index: I,
    sequence_storage: S,
}

impl<I, S> Reference<I, S> where
    I: PatternIndex,
    S: SequenceStorage,
{
    pub fn new(
        sequence_storage: S,
        pattern_index_option: I::Option,
    ) -> Result<Self, I::BuildError> {
        let (concatenated_sequence, target_boundaries) = sequence_storage.get_concatenated_sequence_with_boundaries_of_targets();
        let pattern_index = I::new(concatenated_sequence, pattern_index_option)?;

        Ok(Self {
            target_boundaries,
            pattern_index,
            sequence_storage,
        })
    }
    pub fn get_sequence_storage(&self) -> &S {
        &self.sequence_storage
    }
    pub fn get_pattern_index(&self) -> &I {
        &self.pattern_index
    }
}
