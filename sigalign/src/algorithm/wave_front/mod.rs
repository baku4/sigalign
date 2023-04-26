use crate::core::regulators::Penalty;
use super::AnchorIndex;
use bytemuck::{Pod, Zeroable};

mod match_counter;
use match_counter::{MatchCounter, ForwardMatchCounter, ReverseMatchCounter};
mod fill;
mod backtrace;
pub use backtrace::BackTraceResult;

// Wave Front
#[derive(Debug, Clone)]
pub struct WaveFront {
    pub max_penalty: usize,
    pub end_point: WaveEndPoint,
    pub wave_front_scores: Vec<WaveFrontScore>,
}

#[derive(Debug, Clone)]
pub struct WaveEndPoint {
    pub penalty: usize,
    pub k: Option<i32>,
}

#[derive(Debug, Clone)]
pub struct WaveFrontScore {
    pub max_k: i32,
    pub components_by_k: Vec<Components>, // (-max_k..=max_k)
}

impl WaveFront {
    pub fn new_allocated(
        penalties: &Penalty,
        max_penalty: usize,
    ) -> Self {
        let wave_front_score_count = max_penalty + 1;
        let gap_open_penalty = penalties.o;
        let gap_extend_penalty = penalties.e;

        let mut wave_front_scores: Vec<WaveFrontScore> = Vec::with_capacity(wave_front_score_count);
        let first_wave_front_score = WaveFrontScore::with_max_k(0);

        let optional_penalty_from_one_gap = max_penalty.checked_sub((gap_open_penalty + gap_extend_penalty) as usize);

        match optional_penalty_from_one_gap {
            Some(penalty_from_one_gap) => {
                (0..gap_open_penalty + gap_extend_penalty).for_each(|_| {
                    wave_front_scores.push(first_wave_front_score.clone());
                });
    
                let quot = (penalty_from_one_gap as u32 / gap_extend_penalty) as i32;
                let rem = penalty_from_one_gap as u32 % gap_extend_penalty;
                for max_k in 1..quot+1 {
                    (0..gap_extend_penalty).for_each(|_| {
                        wave_front_scores.push(WaveFrontScore::with_max_k(max_k));
                    });
                };
                (0..rem+1).for_each(|_| {
                    wave_front_scores.push(WaveFrontScore::with_max_k(quot+1));
                });
            },
            None => {
                (0..max_penalty+1).for_each(|_| {
                    wave_front_scores.push(first_wave_front_score.clone());
                });
            },
        }

        Self {
            max_penalty,
            end_point: WaveEndPoint { penalty: 0, k: None },
            wave_front_scores,
        }
    }
    #[inline]
    pub fn is_reached_to_sequence_end(&self) -> bool {
        self.end_point.k.is_some()
    }
}

impl WaveFrontScore {
    // New
    fn with_max_k(max_k: i32) -> Self {
        Self {
            max_k,
            components_by_k: vec![Components::default(); max_k as usize * 2 + 1], //FIXME: use zeroed
        }
    }
    // Get
    pub fn components_of_k(&self, k: i32) -> &Components {
        &self.components_by_k[(self.max_k + k) as usize]
    }
    pub fn m_component_of_k(&self, k: i32) -> &Component {
        &self.components_of_k(k).m
    }
    pub fn i_component_of_k(&self, k: i32) -> &Component {
        &self.components_of_k(k).i
    }
    pub fn d_component_of_k(&self, k: i32) -> &Component {
        &self.components_of_k(k).d
    }
    pub fn components_of_k_checked(&self, k: i32) -> Option<&Components> {
        self.components_by_k.get((self.max_k + k) as usize)
    }
}

// Components
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct Components {
    pub m: Component,
    pub i: Component,
    pub d: Component,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Component {
    pub fr: i32,
    pub deletion_count: u16,
    pub bt: BackTraceMarker,
    pub traversed: bool,
}
unsafe impl Pod for Component {}
unsafe impl Zeroable for Component {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum BackTraceMarker {
    Empty = 0,
    Start = 1,
    FromM = 2,
    FromI = 3,
    FromD = 4,
}
impl Default for Components {
    fn default() -> Self {
        Self {
            m: Component::empty(),
            i: Component::empty(),
            d: Component::empty(),
        }
    }
}
impl Components {
    fn new_start_point(first_fr: i32) -> Self {
        Self {
            m: Component::start_point(first_fr),
            i: Component::empty(),
            d: Component::empty(),
        }
    }
}

impl Component {
    #[inline(always)]
    fn empty() -> Self {
        Self {
            fr: 0,
            deletion_count: 0,
            bt: BackTraceMarker::Empty,
            traversed: false,
        }
    }
    #[inline(always)]
    fn start_point(first_fr: i32) -> Self {
        Self {
            fr: first_fr,
            deletion_count: 0,
            bt: BackTraceMarker::Start,
            traversed: false,
        }
    }
}
