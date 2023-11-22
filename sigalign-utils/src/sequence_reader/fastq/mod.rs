use std::{
    io::Read,
    fs::File,
    path::Path,
    str::Utf8Error,
};

use seq_io::fastq::{
    Reader as SeqIoReader,
    RefRecord as SeqIoRecord, Record,
};

use super::{
    SeqRecord,
    SeqRefRecord,
    IdRecord,
    IdRefRecord,
};

/// The reader of FASTA formatted file
pub struct FastqReader<R: Read> {
    reader: SeqIoReader<R>,
}

pub struct FastqRecord<'a> {
    record: SeqIoRecord<'a>,
}

impl<'a, R: Read> FastqReader<R> {
    pub fn new(reader: R) -> Self {
        let reader = SeqIoReader::new(reader);
        Self {
            reader
        }
    }
    pub fn next(&'a mut self) -> Option<FastqRecord<'a>> {
        if let Some(Ok(seq)) = self.reader.next() {
            Some(FastqRecord {
                record: seq,
            })
        } else {
            None
        }
    }
}
impl FastqReader<File> {
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self, std::io::Error> {
        let reader = SeqIoReader::from_path(path)?;
        Ok(Self {
            reader
        })
    }
}
impl<'a> FastqReader<&'a [u8]> {
    pub fn from_bytes(bytes: &'a [u8]) -> Self {
        Self::new(bytes)
    }
}

impl<'a> SeqRecord for FastqRecord<'a> {
    fn extend_seq_buf(&mut self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(self.record.seq());
    }
}

impl<'a> SeqRefRecord for FastqRecord<'a> {
    fn seq(&self) -> &[u8] {
        self.record.seq()
    }
}

impl<'a> IdRecord for FastqRecord<'a> {
    fn extend_id_buf(&mut self, buf: &mut Vec<u8>) {
        buf.extend(self.record.id_bytes());
    }
    fn extend_id_string(&mut self, buf: &mut String) -> Result<(), Utf8Error> {
        buf.push_str(self.record.id()?);
        Ok(())
    }
}

impl<'a> IdRefRecord for FastqRecord<'a> {
    fn id(&self) -> &[u8] {
        self.record.id_bytes()
    }
    fn id_str(&self) -> Result<&str, Utf8Error> {
        self.record.id()
    }
}
