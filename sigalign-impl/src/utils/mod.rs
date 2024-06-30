#![allow(unused_imports)]
#![allow(dead_code)]

/*!
Utilities.
*/ 
mod sequence_manipulation;
pub use sequence_manipulation::{
    reverse_complement_of_dna,
    reverse_complement_of_rna,
    get_unique_characters_of_sequence,
};
