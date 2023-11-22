use std::{io::{Read, Error}, fs::File, path::Path, str::Utf8Error};

use seq_io::fasta::{
    Reader as SeqIoReader,
    RefRecord as SeqIoRecord, Record,
};

use super::{
    SeqRecord,
    IdRecord,
    IdRefRecord,
};

/// The reader of FASTA formatted file
pub struct FastaReader<R: Read> {
    reader: SeqIoReader<R>,
}

pub struct FastaRecord<'a> {
    record: SeqIoRecord<'a>,
}

impl<'a, R: Read> FastaReader<R> {
    pub fn new(reader: R) -> Self {
        let reader = SeqIoReader::new(reader);
        Self {
            reader
        }
    }
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
impl FastaReader<File> {
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let reader = SeqIoReader::from_path(path)?;
        Ok(Self {
            reader
        })
    }
}

impl<'a> SeqRecord for FastaRecord<'a> {
    fn extend_seq_buf(&mut self, buf: &mut Vec<u8>) {
        self.record.seq_lines().for_each(|s| buf.extend_from_slice(s));
    }
}

impl<'a> IdRecord for FastaRecord<'a> {
    fn extend_id_buf(&mut self, buf: &mut Vec<u8>) {
        buf.extend(self.record.id_bytes());
    }
    fn extend_id_string(&mut self, buf: &mut String) -> Result<(), Utf8Error> {
        buf.push_str(self.record.id()?);
        Ok(())
    }
}

impl<'a> IdRefRecord for FastaRecord<'a> {
    fn id(&self) -> &[u8] {
        self.record.id_bytes()
    }
    fn id_str(&self) -> Result<&str, Utf8Error> {
        self.record.id()
    }
}
