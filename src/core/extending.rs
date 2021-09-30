use super::{Cutoff, Penalties};
use super::{Reference, Query};
use super::{Anchors, Anchor, Estimation, CheckPoints};

mod dwfa;

use std::collections::HashMap;

impl Anchors {
    fn extending_anchors(
        &mut self,
        record_sequence: Query,
        query: Query,
    ) {
        for anchor in &mut self.anchors {
            anchor.extend_right();
        }
    }
}

impl Anchor {
    fn extend_right(&mut self) {
        
    }
}