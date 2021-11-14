use super::{Reference, ReferenceProto, SequenceProvider, Labeling, Writable};

mod file_reader;
mod in_memory;
mod fai_used_fasta;
mod indexed_fasta;

#[cfg(not(target_arch = "wasm32"))]
mod sqlite;

pub use file_reader::FastaReader;
pub use in_memory::InMemoryProvider;
pub use fai_used_fasta::FaiUsedFastaProvider;
pub use indexed_fasta::IndexedFastaProvider;
#[cfg(not(target_arch = "wasm32"))]
pub use sqlite::SqliteProvider;

const A_UTF8: u8 = 65;
const C_UTF8: u8 = 67;
const G_UTF8: u8 = 71;
const T_UTF8: u8 = 84;

fn reverse_complement_of_nucleotide_sequence(sequence: &[u8]) -> Vec<u8> {
    sequence.iter().rev().map(|&character| {
        match character{
            A_UTF8 => T_UTF8,
            C_UTF8 => G_UTF8,
            G_UTF8 => C_UTF8,
            T_UTF8 => A_UTF8,
            _ => character,
        }
    }).collect()
}