use super::{
    SequenceStorage, SequenceBuffer,
    ConcatenatedSequenceWithBoundaries,
};

mod in_memory;
pub use in_memory::InMemoryStorage;
// mod indexed_fasta;