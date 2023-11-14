use crate::core::Sequence;

use sigalign::utils::FastaReader;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

pub struct OneFastaIterator<R: Read> {
    next_index: u32,
    reader: FastaReader<R>,
}

impl<R: Read> Iterator for OneFastaIterator<R> {
    type Item = Sequence;

    fn next(&mut self) -> Option<Self::Item> {
        match self.reader.next() {
            Some((label, seq)) => {
                let sequence = Sequence {
                    index: self.next_index,
                    id: Some(label),
                    inner: seq,
                };
                self.next_index += 1;
                Some(sequence)
            },
            None => None
        }
    }
}

impl OneFastaIterator<File> {
    pub fn from_path(path: PathBuf) -> Result<Self, std::io::Error> {
        let reader = FastaReader::from_path(path)?;
        Ok(Self {
            next_index: 0,
            reader,
        })
    }
}

impl<'a> OneFastaIterator<&'a [u8]> {
    pub fn from_bytes(bytes: &'a [u8]) -> Result<Self, std::io::Error> {
        let reader = FastaReader::from_bytes(bytes);
        Ok(Self {
            next_index: 0,
            reader,
        })
    }
}

