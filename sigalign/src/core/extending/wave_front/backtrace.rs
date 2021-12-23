use super::Penalties;
use super::Sequence;
use super::{AlignmentOperation, AlignmentType};
use super::{WaveFront, EndPoint, WaveFrontScore, Components, Component, BackTraceMarker, MatchCounter};

impl WaveFront {
    fn is_aligned_to_end(&self) -> bool {
        self.end_point.have_k()
    }
    fn backtrace_from_point(
        &self,

    ) {
        
    }
}

impl EndPoint {
    fn have_k(&self) -> bool {
        self.k.is_some()
    }
}

enum ComponentType {
    M,
    I,
    D,
}