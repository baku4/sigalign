use super::{Reference, Aligner};

const NUCLEOTIDES: [u8; 4] = [b'A', b'C', b'G', b'T'];

mod target;
mod query;
pub use target::get_sample_target_as_fasta_string;
pub use query::get_sample_query;
