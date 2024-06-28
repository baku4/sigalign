use sigalign_core::results::QueryAlignment;
use super::Aligner;
use crate::reference::Reference;

impl Aligner {
    /// Align a query to the reference.
    pub fn alignment(
        &mut self,
        query: &[u8],
        reference: &Reference,
    ) -> QueryAlignment {
        self.dynamic_aligner.alignment(
            query,
            reference.as_ref(),
            &mut self.sequence_buffer,
            reference.get_full_sorted_target_indices(),
        )
    }
}
