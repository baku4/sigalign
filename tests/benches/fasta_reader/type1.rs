use std::{fs::File, path::PathBuf};

use seq_io::fasta::{
    Reader as SeqIoReader,
};

pub struct FastaReader1 {
    reader: SeqIoReader<File>,
}
pub trait SeqReader1<'a> {
    fn fill_next(&mut self, buf: &'a mut Vec<u8>) -> Option<&'a [u8]>;
}

impl<'a> SeqReader1<'a> for FastaReader1 {
    fn fill_next(&mut self, buf: &'a mut Vec<u8>) -> Option<&'a [u8]> {
        buf.clear();
        if let Some(Ok(record)) = self.reader.next() {
            record.seq_lines().for_each(|x| buf.extend(x));
            Some(buf)
        } else {
            None
        }
    }
}
impl FastaReader1 {
    pub fn new(fasta_path: PathBuf) -> Self {
        let seq_io_reader =  SeqIoReader::from_path(fasta_path).expect("Error reading fasta file");
        Self { reader: seq_io_reader }
    }
    pub fn new_repeat(fasta_path: PathBuf, repeat: usize) -> Vec<Self> {
        (0..repeat).map(|_| Self::new(fasta_path.clone())).collect()
    }
}

pub fn count_seq_len_with_fasta_readers_1(
    fasta_readers: Vec<FastaReader1>,
) -> usize {
    let mut buf_seq: Vec<u8> = Vec::new();
    let mut count = 0;
    for mut fasta_reader in fasta_readers {
        while let Some(seq) = fasta_reader.fill_next(&mut buf_seq) {
            count += seq.len();
        }
    }
    count
}
