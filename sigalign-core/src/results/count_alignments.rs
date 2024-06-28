use super::{
    QueryAlignment,
    TargetAlignment,
};

impl QueryAlignment {
    pub fn count_alignments(&self) -> usize {
        self.0.iter().map(|x| x.count_alignments()).sum()
    }
}
impl TargetAlignment {
    pub fn count_alignments(&self) -> usize {
        self.alignments.len()
    }
}
