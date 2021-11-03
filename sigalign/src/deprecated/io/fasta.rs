use std::path::Path;
use std::{fs::read, io::{Read, Write, BufReader}};
use std::fs::File;

/*  FASTA & FASTQ  */
use bio::io::{fasta, fastq};
use bio::io::fasta::IndexedReader;
// From `Rust-Bio` crate  
// https://crates.io/crates/bio  

// fasta
pub type FastaReader = fasta::Reader<BufReader<File>>;
pub type FastaRecords = fasta::Records<BufReader<File>>;
pub fn fasta_records(file_path: &str) -> FastaRecords {
    fasta::Reader::from_file(file_path).expect("Errored in reading fasta").records()
}

// fastq
pub type FastqReader = fastq::Reader<BufReader<File>>;
pub type FastqRecords = fastq::Records<BufReader<File>>;
pub fn fastq_records(file_path: &str) -> FastqRecords {
    fastq::Reader::from_file(file_path).expect("Errored in reading fastq").records()
}

// fasta with index
//TODO: 