//! Sequence reader module
//! This module provides a generic interface for reading sequence files.
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
}
pub trait IdRefRecord {
    fn id(&self) -> &[u8];
}
