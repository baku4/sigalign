use super::{
    AlignmentResult,
    TargetAlignmentResult,
    fasta::{
        FastaAlignmentResult,
        ReadAlignmentResult,
    },
    labeled::{
        LabeledAlignmentResult,
        LabeledTargetAlignmentResult,
    }
};

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
impl LabeledAlignmentResult {
    pub fn result_counts(&self) -> usize {
        self.0.iter().map(|x| x.result_counts()).sum()
    }
}
impl LabeledTargetAlignmentResult {
    pub fn result_counts(&self) -> usize {
        self.alignments.len()
    }
}