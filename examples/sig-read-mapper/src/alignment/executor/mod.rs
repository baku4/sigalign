use sigalign::Aligner;

use super::AlignmentConfig;

pub struct AlignmentExecutor {
    sig_aligners: Vec<Option<Aligner>>,
}

impl AlignmentExecutor {
    pub fn new(config: &AlignmentConfig) -> Self {
        // TODO:
        Self {
            sig_aligners: Vec::new(),
        }
    }
    pub fn perform_alignment(&mut self) {
        // TODO:
    }
}