use super::Penalties;
use super::Sequence;
use super::{AlignmentOperation, AlignmentType};

type MatchCounter<'a> = &'a dyn Fn(Sequence, Sequence, usize, usize) -> i32;

mod fill;
mod backtrace;

// Wave Front

#[derive(Debug)]
pub struct WaveFront {
    pub max_score: usize,
    pub end_point: EndPoint,
    pub wave_front_scores: Vec<WaveFrontScore>,
}

#[derive(Debug)]
pub struct EndPoint {
    score: usize,
    k: Option<i32>,
}

#[derive(Debug, Clone)]
pub struct WaveFrontScore {
    pub max_k: i32,
    pub components_by_k: Vec<Components>, // (-max_k..=max_k)
}

impl WaveFront {
    pub fn new_allocated(
        penalties: &Penalties,
        max_score: usize,
    ) -> Self {
        let wave_front_score_count = max_score + 1;
        let gap_open_penalty = penalties.o;
        let gap_extend_penalty = penalties.e;

        let mut wave_front_scores: Vec<WaveFrontScore> = Vec::with_capacity(wave_front_score_count);

        let first_wave_front_score = WaveFrontScore::with_max_k(0);

        let optional_penalty_from_one_gap = max_score.checked_sub(gap_open_penalty + gap_extend_penalty);

        match optional_penalty_from_one_gap {
            Some(penalty_from_one_gap) => {
                (0..gap_open_penalty + gap_extend_penalty).for_each(|_| {
                    wave_front_scores.push(first_wave_front_score.clone());
                });
    
                let quot = (penalty_from_one_gap / gap_extend_penalty) as i32;
                let rem = penalty_from_one_gap % gap_extend_penalty;
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
                (0..max_score+1).for_each(|_| {
                    wave_front_scores.push(first_wave_front_score.clone());
                });
            },
        }

        Self {
            max_score: max_score,
            end_point: EndPoint { score: 0, k: None },
            wave_front_scores,
        }
    }
    fn clear_components(&mut self) {
        // TODO:
    }
}

impl WaveFrontScore {
    // New
    fn with_max_k(max_k: i32) -> Self {
        Self {
            max_k,
            components_by_k: Vec::with_capacity(max_k as usize * 2 + 1),
        }
    }
    // Get
    fn range_of_k(&self) -> Vec<i32> {
        (-self.max_k..=self.max_k).collect()
    }
    fn components_of_k(&self, k: i32) -> &Components {
        &self.components_by_k[(self.max_k + k) as usize]
    }
    fn m_component_of_k(&self, k: i32) -> &Component {
        &self.components_of_k(k).m
    }
    fn i_component_of_k(&self, k: i32) -> &Component {
        &self.components_of_k(k).i
    }
    fn d_component_of_k(&self, k: i32) -> &Component {
        &self.components_of_k(k).d
    }
    fn components_of_k_checked(&self, k: i32) -> Option<&Components> {
        self.components_by_k.get((self.max_k + k) as usize)
    }
    // Reset
}



// Components


#[repr(C)]
#[derive(Debug, Clone)]
pub struct Components {
    m: Component,
    i: Component,
    d: Component,
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct Component {
    pub fr: i32,
    pub deletion_count: u16,
    pub bt: BackTraceMarker,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum BackTraceMarker {
    Empty,
    Start,
    FromM,
    FromI,
    FromD,
}

impl Components {
    fn new_empty() -> Self {
        Self {
            m: Component::empty(),
            i: Component::empty(),
            d: Component::empty(),
        }
    }
    fn new_start_point(first_fr: i32) -> Self {
        Self {
            m: Component::start_point(first_fr),
            i: Component::empty(),
            d: Component::empty(),
        }
    }
}

impl Component {
    fn empty() -> Self {
        Self {
            fr: 0,
            deletion_count: 0,
            bt: BackTraceMarker::Empty,
        }
    }
    fn start_point(first_fr: i32) -> Self {
        Self {
            fr: first_fr,
            deletion_count: 0,
            bt: BackTraceMarker::Start,
        }
    }
}

//TODO: Apply SIMD
fn consecutive_match_forward(ref_seq: &[u8], qry_seq: &[u8], v: usize, h: usize) -> i32 {
    let mut fr_to_add: i32 = 0;
    for (v1, v2) in qry_seq[v..].iter().zip(ref_seq[h..].iter()) {
        if *v1 == *v2 {
            fr_to_add += 1;
        } else {
            return fr_to_add
        }
    }
    fr_to_add
}
fn consecutive_match_reverse(ref_seq: &[u8], qry_seq: &[u8], v: usize, h: usize) -> i32 {
    let mut fr_to_add: i32 = 0;
    for (v1, v2) in qry_seq[..qry_seq.len()-v].iter().rev().zip(ref_seq[..ref_seq.len()-h].iter().rev()) {
        if *v1 == *v2 {
            fr_to_add += 1;
        } else {
            return fr_to_add
        }
    }
    fr_to_add
}
