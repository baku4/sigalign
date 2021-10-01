use super::{Cutoff, Penalties};
use super::{Reference, Sequence};
use super::{Anchors, Anchor, Estimation, CheckPoints};

mod dwfa;

use std::collections::HashMap;

impl Anchors {
    pub fn extend(
        &mut self,
        record_sequence: Sequence,
        query: Sequence,
    ) {
        for anchor in &mut self.anchors {
            anchor.extend_right(record_sequence, query);
        }
    }
}

impl Anchor {
    fn extend_right(
        &mut self,
        record_sequence: Sequence,
        query: Sequence,
    ) {
        if !self.dropped {
            let record_sequence = &record_sequence[self.record_position..];
        }
    }
}