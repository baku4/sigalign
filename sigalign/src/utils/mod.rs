/*!
Utilities
*/ 
mod file_reader;
pub use file_reader::FastaReader;

mod sequence_manipulation;
pub use sequence_manipulation::{
    reverse_complement_of_dna,
    reverse_complement_of_rna,
};

// mod path_reader;
// pub use path_reader::{
//     path_to_byte,
//     byte_to_pathbuf,
// };
