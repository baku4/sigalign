/*!
Provides the `Reference` struct, a database for multiple targeted sequences.

## Features

- The `Reference` struct operates as a central repository for multiple target sequences, primarily for use by the `Aligner` to perform alignments.
- During alignment, `Reference` remains immutable. It manages sequences through the `SequenceBuffer` defined in `SequenceStorage`.
- The search range for the `Reference` can be tailored to specific needs.

## Architecture

The `Reference` encapsulates types conforming to the `SequenceStorage` and `PatternIndex` traits. Construction of a `Reference` requires specification of structs that implement these traits. While SigAlign offers default implementations in the 'sequence_storage' and 'pattern_index' modules, custom implementations are supported.

Upon the `Reference` structure's creation, direct access to `SequenceStorage` and `PatternIndex` implementations is restricted, with management handled by the `Aligner`.

### Trait `SequenceStorage`

`SequenceStorage` is responsible for returning a target sequence when provided with its index. SigAlign remains agnostic to the storage and retrieval methods for sequences; these could be held in memory, stored in a file, or located in a remote physical location accessible over a network.

`SequenceStorage` is designed to fetch a target sequence based on a given target index. SigAlign remains indifferent to the sequence storage and retrieval methods, which could be memory, file-based, or located in a remote physical space connected over a network.

### Trait `PatternIndex`

`PatternIndex` accepts pattern bytes and returns the indices of the targets exactly matching the pattern. The performance of `PatternIndex` significantly influences overall performance, and it can vary widely based on implementation details. Thus, `PatternIndex` should be defined differently according to use cases, considering factors like the maximum number of character types that can be indexed, the length of the targets, and the characteristics of the input query.

## Usage

### (1) Constructing `Reference` with `SequenceStorage`

```rust
use sigalign::reference::{
    Reference,
    sequence_storage::in_memory::InMemoryStorage,
    pattern_index::lfi::{Lfi32B2V64, LfiOption},
};

// (1) Define the SequenceStorage
let mut sequence_storage = InMemoryStorage::new();
sequence_storage.add_target(
    "target_1",
    b"AAAA...AAA",
);
sequence_storage.add_target(
    "target_2",
    b"CCCC...CCC",
);

// (2) Set options for PatternIndex
let pattern_index_option = LfiOption::new(2, 4, true);

// (3) Construct Reference
let reference = Reference::<Lfi32B2V64, InMemoryStorage>::new(
    sequence_storage,
    pattern_index_option,
).unwrap();
```

### (2) Performing Alignment

#### Use `Aligner`

```rust
let result = aligner.align_query(
    &reference,
    b"AA...CC",
);

let result = aligner.align_fasta_file(
    &reference,
    "FASTA_FILE_PATH",
);
```

#### Directly manipulate `SequenceBuffer`

```rust
let mut sequence_buffer = reference.get_sequence_buffer();
for query in [b"AA...CC", b"GG...TT"] {
    aligner.alignment(
        &reference,
        &mut sequence_buffer,
        query,
    );
}
```

### (3) Additional Features

#### Adjusting search range

```rust
let mut reference = reference;
// Perform alignment only on targets with index 0 and 1
reference.set_search_range(vec![0, 1]).unwrap();
```

#### Saving and Loading `Reference`

```rust
use sigalign::reference::extensions::Serialize;
// Save
reference.save_to(&mut buffer).unwrap();
// Load
let reference = Reference::<Lfi32B2V64, InMemoryStorage>::load_from(&buffer[..]).unwrap();
```
*/
// mod sequence_type;
// use sequence_type::SequenceType;
pub mod pattern_index;
use pattern_index::{
    PatternIndex,
    ConcatenatedSequenceWithBoundaries,
    PatternIndexBuildError,
};
pub mod sequence_storage;
use sequence_storage::SequenceStorage;

/// A database for multiple targeted sequences.
#[derive(Debug)]
pub struct Reference<I, S> where
    I: PatternIndex,
    S: SequenceStorage,
{
    search_range: Vec<u32>,
    pattern_index: I,
    pub(crate) sequence_storage: S,
}

impl<I, S> Reference<I, S> where
    I: PatternIndex,
    S: SequenceStorage,
{
    pub fn new(
        sequence_storage: S,
        pattern_index_option: I::Option,
    ) -> Result<Self, ReferenceBuildError> {
        let concatenated_sequence_with_boundaries = sequence_storage.get_concatenated_sequence_with_boundaries();
        let pattern_index = I::new(
            concatenated_sequence_with_boundaries,
            pattern_index_option,
        )?;
        let num_targets = sequence_storage.num_targets();
        let search_range: Vec<u32> = (0..num_targets).collect();

        Ok(Self {
            search_range,
            pattern_index,
            sequence_storage,
        })
    }
}

use thiserror::Error;
/// Enumerates possible errors encountered when constructing a `Reference`.
#[derive(Debug, Error)]
pub enum ReferenceBuildError {
    #[error(transparent)]
    PatternIndexBuildError(#[from] PatternIndexBuildError),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
}

// Features
mod pattern_search;
mod debug;
mod set_search_range;
pub use set_search_range::SetSearchRangeError;
// Extensions
pub mod extensions;
