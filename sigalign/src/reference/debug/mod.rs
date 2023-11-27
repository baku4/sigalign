use std::fmt::Debug;

use super::Reference;
use sigalign_core::reference::extensions::EstimateSize;

impl Debug for Reference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Reference")
            .field("num_targets", &self.as_ref().num_targets())
            .field("estimated_size_in_byte", &self.as_ref().serialized_size())
            .finish()
    }
}
