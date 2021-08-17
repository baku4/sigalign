use bio::io::fasta::Sequence;

use crate::alignment::Aligner;

/// Location of the database:
/// (index of sequence, start position of pattern)
pub struct Location {
    pub index: usize,
    pub position: usize,
}

/// Database
pub trait Database {
    fn search(&self, aligner: &Aligner, query: &[u8]);
    fn locate(&self, query: &[u8]) -> Location;
}

/// Records of Sequences
trait SequenceRecords<'a> {
    fn len(&self) -> usize;
    fn sequence(&self, index: usize) -> &'a [u8];
    fn label(&self, index: usize) -> &'a str;
}

struct SearchResult {
    label: String,
    sequence: Vec<u8>,
}
