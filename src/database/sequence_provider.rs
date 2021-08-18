use crate::utils::get_reverse_complement;
use crate::io::fasta::{FastaRecords, fasta_records};
use crate::io::index::{read_lt_fm_index_from_file_path, read_lt_fm_index_from_inferred_path};
use super::{SequenceProvider, AccumulatedLength};

use serde::{Serialize, Deserialize};

mod memory;
mod fasta;
mod sqlite;

pub use memory::OnMemoryProvider;
pub use fasta::FastaProvider;
pub use sqlite::SqliteProvider;