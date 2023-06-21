use super::{
    AlignmentResult,
    TargetAlignmentResult,
};
use super::fasta::{
    FastaAlignmentResult,
    ReadAlignmentResult,
};

impl FastaAlignmentResult {
    pub fn result_counts(&self) -> usize {
        self.0.iter().map(|x| x.result_counts()).sum()
    }
}
impl ReadAlignmentResult {
    pub fn result_counts(&self) -> usize {
        self.result.result_counts()
    }
}
impl AlignmentResult {
    pub fn result_counts(&self) -> usize {
        self.0.iter().map(|x| x.result_counts()).sum()
    }
}
impl TargetAlignmentResult {
    pub fn result_counts(&self) -> usize {
        self.alignments.len()
    }
}
