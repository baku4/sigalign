use super::{
    AlignmentLabeledResult,
    FastaAlignmentLabeledResult,
};

pub struct Result {
    inner: Option<InnerResult>,
}

impl Default for Result {
    fn default() -> Self {
        Self {
            inner: None,
        }
    }
}

pub enum InnerResult {
    Query(AlignmentLabeledResult),
    Fasta(FastaAlignmentLabeledResult),
}
