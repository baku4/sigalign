use super::{
    AlignmentResult,
    TargetAlignmentResult,
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