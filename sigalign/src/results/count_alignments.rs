use super::{
    FastaAlignmentResult,
    ReadAlignmentResult,
};

impl FastaAlignmentResult {
    pub fn count_alignments(&self) -> usize {
        self.0.iter().map(|read| read.count_alignments()).sum()
    }
}
impl ReadAlignmentResult {
    pub fn count_alignments(&self) -> usize {
        self.result.count_alignments()
    }
}
