use super::{Reference, ReferenceProtoDep, SequenceProvider, Labeling, Writable};
use super::{FastaReader, reverse_complement_of_nucleotide_sequence};

mod in_memory;
mod fasta_with_fai;
mod indexed_fasta;
#[cfg(not(target_arch = "wasm32"))]
mod sqlite;

pub use in_memory::InMemoryProvider;
pub use fasta_with_fai::FastaWithFaiProvider;
pub use indexed_fasta::IndexedFastaProvider;
#[cfg(not(target_arch = "wasm32"))]
pub use sqlite::SqliteProvider;
