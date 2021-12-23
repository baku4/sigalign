use super::Penalties;
use super::Sequence;
use super::{AlignmentOperation, AlignmentType};
use super::{Extension, WaveFront, EndPoint, WaveFrontScore, Components, Component, BackTraceMarker, MatchCounter};

impl WaveFront {
    pub fn backtrace_from_the_end(&self, penalties: &Penalties) -> Option<Extension> {
        match self.end_point.k {
            Some(k) => {
                let last_score = self.end_point.score;
                let index_of_component = self.wave_front_scores[last_score].max_k as usize + last_score;
                Some(self.backtrace_from_point(last_score, index_of_component, penalties))
            },
            None => {
                None
            },
        }
    }
    pub fn backtrace_from_point(
        &self,
        score: usize,
        index_of_component: usize,
        penalties: &Penalties,
    ) -> Extension {
        // TODO:
    }
}

enum ComponentType {
    M,
    I,
    D,
}