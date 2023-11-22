//! Sequence reader module
//! This module provides a generic interface for reading sequence files.
use std::str::Utf8Error;

pub mod fasta;
pub mod fastq;

pub mod decompress;

pub trait SeqRecord {
    fn extend_seq_buf(&mut self, buf: &mut Vec<u8>);
}
pub trait SeqRefRecord {
    fn seq(&self) -> &[u8];
}
pub trait IdRecord {
    fn extend_id_buf(&mut self, buf: &mut Vec<u8>);
    fn extend_id_string(&mut self, buf: &mut String) -> Result<(), Utf8Error>;
}
pub trait IdRefRecord {
    fn id(&self) -> &[u8];
    fn id_str(&self) -> Result<&str, Utf8Error>;
}
