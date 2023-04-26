use std::path::Path;
use std::io::Read;
use std::fs::File;

use seq_io::fasta::{
    Reader as SeqIoReader,
    Record,
};

/// The reader of FASTA formatted file
pub struct FastaReader<R: Read> {
    reader: SeqIoReader<R>,
}

impl<R: Read> FastaReader<R> {
    pub fn new(reader: R) -> Self {
        let reader = SeqIoReader::new(reader);
        Self {
            reader
        }
    }
}
impl FastaReader<File> {
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self, std::io::Error> {
        let reader = SeqIoReader::from_path(path)?;
        Ok(Self {
            reader
        })
    }
}
impl<'a> FastaReader<&'a [u8]> {
    pub fn from_bytes(bytes: &'a [u8]) -> Self {
        Self::new(bytes)
    }
}
impl<R: Read> Iterator for FastaReader<R> {
    type Item = (String, Vec<u8>);

    fn next(&mut self) -> Option<Self::Item> {
        match self.reader.next() {
            Some(Ok(seq)) => {
                Some((
                    String::from_utf8(seq.id_bytes().to_vec()).unwrap(),
                    seq.to_owned_record().seq,
                ))
            },
            _ => {
                None
            },
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