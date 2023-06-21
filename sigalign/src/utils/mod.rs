/*!
Utilities.
*/ 
mod file_reader;
pub use file_reader::FastaReader;

mod sequence_manipulation;
pub use sequence_manipulation::{
    reverse_complement_of_dna,
    reverse_complement_of_rna,
    get_unique_characters_of_sequence,
};

mod functions;
pub use functions::{
    calculate_max_pattern_size,
};
