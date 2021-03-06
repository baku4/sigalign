use crate::{Result, error_msg};

use std::path::Path;
use std::{fs::read, io::{Read, Write, BufReader}};
use std::fs::File;

use bio::io::{fasta, fastq};
use bio::io::fasta::IndexedReader;

pub struct FastaReader<R: Read> {
    records: fasta::Records<BufReader<R>>
}

impl<R: Read> FastaReader<R> {
    pub fn new(reader: R) -> Self {
        let reader = fasta::Reader::new(reader);

        Self {
            records: reader.records()
        }
    }
}
impl FastaReader<File> {
    pub fn from_file_path<P: AsRef<Path> + std::fmt::Debug>(file_path: P) -> Result<Self> {
        let reader = fasta::Reader::from_file(file_path)?;

        Ok(
            Self {
                records: reader.records()
            }
        )
    }
}
impl<'a> FastaReader<&'a [u8]> {
    pub fn from_bytes(bytes: &'a [u8]) -> Self {
        Self::new(bytes)
    }
}

// TODO: Change fasta reader to new one
impl<R: Read> Iterator for FastaReader<R> {
    type Item = (String, Vec<u8>);

    fn next(&mut self) -> Option<Self::Item> {
        match self.records.next() {
            Some(record) => {
                match record {
                    Ok(record) => {
                        Some((record.id().to_string(), record.seq().to_vec()))
                    },
                    Err(_) => {
                        None
                    }
                }
            },
            None => {
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reading_fasta_from_bytes() {
        let fasta_bytes = b">text\nAGCGTTTTATTACCTTTT";

        let mut fasta_reader = FastaReader::from_bytes(fasta_bytes);

        let (label, seq) = fasta_reader.next().unwrap();

        assert_eq!(label, "text");
        assert_eq!(&seq, b"AGCGTTTTATTACCTTTT");
    }
}