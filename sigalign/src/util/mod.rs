use super::{Result, error_msg};

mod file_reader;
pub use file_reader::FastaReader;

mod sequence_manipulation;
pub use sequence_manipulation::reverse_complement_of_nucleotide_sequence;

mod error_encoding;
pub use error_encoding::transform_res_type;
