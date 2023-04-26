use super::{
    AlignmentResult,
    RecordAlignmentResult,
};
use super::{
    FastaAlignmentResult,
    ReadAlignmentResult,
    FastaAlignmentLabeledResult,
    ReadAlignmentLabeledResult,
    AlignmentLabeledResult,
    RecordAlignmentLabeledResult,
};

impl FastaAlignmentResult {
    pub fn result_counts(&self) -> usize {
        self.0.iter().map(|x| x.result_counts()).sum()
    }
}
impl FastaAlignmentLabeledResult {
    pub fn result_counts(&self) -> usize {
        self.0.iter().map(|x| x.result_counts()).sum()
    }
}
impl ReadAlignmentResult {
    pub fn result_counts(&self) -> usize {
        self.result.result_counts()
    }
}
impl ReadAlignmentLabeledResult {
    pub fn result_counts(&self) -> usize {
        self.result.result_counts()
    }
}
impl AlignmentResult {
    pub fn result_counts(&self) -> usize {
        self.0.iter().map(|x| x.result_counts()).sum()
    }
}
impl AlignmentLabeledResult {
    pub fn result_counts(&self) -> usize {
        self.0.iter().map(|x| x.result_counts()).sum()
    }
}
impl RecordAlignmentResult {
    pub fn result_counts(&self) -> usize {
        self.alignments.len()
    }
}
impl RecordAlignmentLabeledResult {
    pub fn result_counts(&self) -> usize {
        self.alignments.len()
    }
}
