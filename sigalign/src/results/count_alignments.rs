use super::{
    LabeledQueryAlignment,
    LabeledTargetAlignment,
};

impl LabeledQueryAlignment {
    pub fn count_alignments(&self) -> usize {
        self.0.iter().map(|lta| lta.count_alignments()).sum()
    }   
}
impl LabeledTargetAlignment {
    pub fn count_alignments(&self) -> usize {
        self.alignments.len()
    }
}
