use crate::{Result, error_msg};

use std::path::Path;
use std::{fs::read, io::{Read, Write, BufReader}};
use std::fs::File;

use bio::io::{fasta, fastq};
use bio::io::fasta::IndexedReader;

pub struct FastaReader {
    records: fasta::Records<BufReader<File>>,
}
impl FastaReader {
    pub fn from_file_path<P: AsRef<Path> + std::fmt::Debug>(file_path: P) -> Result<Self> {
        let reader = fasta::Reader::from_file(file_path)?;

        Ok(
            Self {
                records: reader.records()
            }
        )
    }
}

impl Iterator for FastaReader {
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