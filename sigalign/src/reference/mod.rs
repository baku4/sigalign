/*! Configurations for [Reference] generation

`Reference` is a bundle of alignment target sequences.

# (1) Parameters to generate `Reference`
1. [SequenceType]
    - Definition of type of Sequence
        - **Four** types are supported.
            - Nucleotide only
                - A, C, G, T
            - Nucleotide with noise
                - A, C, G, T + One unspecified character
            - Aminoacid only
                - A, C, D, E, F, G, H, I, K, L, M, N, P, Q, R, S, T, V, W, Y
            - Aminoacid with noise
                - A, C, D, E, F, G, H, I, K, L, M, N, P, Q, R, S, T, V, W, Y + One unspecified character
    - `SequenceType` is optional parameter for `Reference` generation because it can be inferred automatically. If passed, however, the `Reference` generation is faster because the inferring process can be skipped.
2. [LtFmIndexConfig]
    - Configurations for `fm-index` data structure of [`lt-fm-index` crate](https://crates.io/crates/lt-fm-index).
    - There are **three** options.
        - BWT size
            - Whether the size of the BWT block is 64 or 128.
            - Default is using 64 sized block.
            - Using 128 sized block lowers the memory usage of the `lt-fm-index` but slows the algorithm.
        - Sampling ratio
            - Sampling ratio for suffix array in `lt-fm-index`.
            - Default is 2.
            - The larger the value, the lower the memory usage, but the slower the algorithm.
        - Kmer size for lookup table
            - Kmer size for lookup table in `lt-fm-index`.
                - Lookup table is pre-calculated count array.
                - Using k-sized lookup table can skip first k LF-mapping operations.
            - Default value is depending on `lt-fm-index` crate.
                - [Configuration for `lt-fm-index`](https://docs.rs/lt-fm-index/0.4.0/lt_fm_index/struct.LtFmIndexConfig.html)
            - The larger the value, the larger the memory usage, but the faster the algorithm.
            - Since memory usage increases exponentially with the number of characters supported by the sequence type, it is not recommended to increase this value too much.
    - `LtFmIndexConfig` is optional parameter for `Reference` generation. If `LtFmIndexConfig` is not passed, the default config is used.
3. [SequenceProvider]
    - `SequenceProvider` is `trait` to provide sequence to `Reference`.
    - It require **two** methods.
        - `total_record_count` to get the number of records.
        - `sequence_of_record` to get sequence from index of record.
    - Method of `joined_sequence_and_accumulated_lengths` can be overrode for better performance.
        - `joined_sequence_and_accumulated_lengths` supply two vectors in tuple.
            - The "joined_sequence" means the sequence of concatenated sequences of all record.
            - The "accumulated_lengths" means the accumulated sequence lengths from 0 to the sum of the lengths of all sequences.
            - For examples, if there are three records with "ATT", "CC", "GGGG", the "joined_sequence" is "ATTCCGGGG" and the "accumulated_lengths" is [0, 3, 5, 9].
        - Since this "joined_sequence" can be very large in size (because there is all sequence), the strategy of memory allocation can be different for each sequence provider. For example, if the length of the entire sequence can be known in advance, allocating whole memory at once is faster than summing up each sequence.
        - Therefore, according to the size of the entire sequence and the characteristics of the sequence provider, whether to override this method or not can be determined by user.
    - `SequenceProvider` is mutable inside the `Reference`.
        - A buffer or pointer may be required.

# (2) Search range
- The search range is a list(`Vec` in Rust) of indexes of records to be searched.
- Search range can be set after `Reference` generation.
- When a reference is generated, it is set for the entire record (0..the number of records).
*/

use crate::{Result, error_msg};
use crate::core::{
	Penalties, PRECISION_SCALE, Cutoff, MinPenaltyForPattern,
	AlignmentResult, RecordAlignmentResult, AnchorAlignmentResult, AlignmentPosition, AlignmentOperation, AlignmentCase,
    Sequence,
    ReferenceInterface, SequenceBuffer, PatternLocation,
};
use crate::util::{FastaReader, reverse_complement_of_nucleotide_sequence};
use crate::{Deserialize, Serialize};
use std::collections::HashMap;
use std::marker::PhantomData;

// Traits implemented by structures
mod requirement;
pub use requirement::{
    Serializable, SizeAware,
    Divisible,
};

// Common data structures for Reference
mod structure;
pub use structure::{
    SequenceType, JoinedSequence, PatternFinder,
};

// Features for Reference
mod feature;
pub use feature::{
    // For sequence provider
    LabelProvider, ReverseComplement,
};

/// Definition and interfaces for `SequenceProvider`
pub mod sequence_provider;
pub use sequence_provider::SequenceProvider;

/// A bundle of alignment target sequences.
#[derive(Debug)]
pub struct Reference<S> where
    S: SequenceProvider,
{
    pub sequence_type: SequenceType,
    pub pattern_finder: PatternFinder,
    pub target_record_index: Vec<u32>,
    pub sequence_provider: S,
}

impl<S> Reference<S> where
    S: SequenceProvider,
{
    pub(crate) fn new(
        sequence_type: SequenceType,
        pattern_finder: PatternFinder,
        target_record_index: Vec<u32>,
        sequence_provider: S
    ) -> Self {
        Self {
            sequence_type,
            pattern_finder,
            target_record_index,
            sequence_provider,
        }
    }
}
