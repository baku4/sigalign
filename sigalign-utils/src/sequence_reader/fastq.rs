use std::{fs::File, path::PathBuf, io::Read};

use seq_io::fasta::{
    Reader as SeqIoReader, RefRecord,
};

pub struct FastaReader2 {
    reader: SeqIoReader<File>,
}

pub struct FastaRecord<'a> {
    record: RefRecord<'a>,
}

trait SeqRecord {
    fn fill_buf(&mut self, buf: &mut Vec<u8>);
}

impl<'a> FastaReader2 {
    pub fn next(&'a mut self) -> Option<FastaRecord<'a>> {
        if let Some(Ok(seq)) = self.reader.next() {
            Some(FastaRecord {
                record: seq,
            })
        } else {
            None
        }
    }
}

impl<'a> SeqRecord for FastaRecord<'a> {
    fn fill_buf(&mut self, buf: &mut Vec<u8>) {
        buf.clear();
        self.record.seq_lines().for_each(|x| buf.extend(x));
    }
}


pub fn count_seq_len_with_fasta_reader_2(
    mut fasta_reader: FastaReader2,
) -> usize {
    let mut buf_seq: Vec<u8> = Vec::new();
    let mut count = 0;

    while let Some(mut record) = fasta_reader.next() {
        record.fill_buf(&mut buf_seq);
        count += buf_seq.len();
    }
    count
}
