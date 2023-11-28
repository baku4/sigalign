use super::{
    AlignmentResult,
    TargetAlignmentResult,
    labeled::{
        LabeledAlignmentResult,
        LabeledTargetAlignmentResult,
    }
};

impl AlignmentResult {
    pub fn count_alignments(&self) -> usize {
        self.0.iter().map(|x| x.count_alignments()).sum()
    }
}
impl TargetAlignmentResult {
    pub fn count_alignments(&self) -> usize {
        self.alignments.len()
    }
}
impl LabeledAlignmentResult {
    pub fn count_alignments(&self) -> usize {
        self.0.iter().map(|x| x.count_alignments()).sum()
    }
}
impl LabeledTargetAlignmentResult {
    pub fn count_alignments(&self) -> usize {
        self.alignments.len()
    }
}