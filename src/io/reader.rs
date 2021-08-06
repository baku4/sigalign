use std::{fs::read, io::BufReader};
// From `Rust-Bio` crate  
// https://crates.io/crates/bio  

use bio::io::fasta;
use std::fs::File;

pub type FastaReader = fasta::Reader<BufReader<File>>;
pub type FastaRecords = fasta::Records<BufReader<File>>;

pub fn fasta_records(file_path: &str) -> FastaRecords {
    fasta::Reader::from_file(file_path).expect("Errored in reading fasta").records()
}