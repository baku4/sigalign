// Dropoff Wave Front Algorithm
use super::Penalties;
use super::Sequence;

type MatchCounter<'a> = &'a dyn Fn(Sequence, Sequence, usize, usize) -> i32;

#[derive(Debug)]
pub struct DropoffWaveFront<C: Component> {
    pub last_score: usize,
    pub last_k: Option<i32>,
    pub wave_front_scores: Vec<WaveFrontScore<C>>,
}
impl<C: Component> DropoffWaveFront<C> {
    pub fn aligned_forward(
        ref_seq: Sequence,
        qry_seq: Sequence,
        penalties: &Penalties,
        spare_penalty: usize,
    ) -> Self {
        Self::new_aligned(ref_seq, qry_seq, penalties, spare_penalty, &consecutive_match_forward)
    }
    pub fn aligned_reverse(
        ref_seq: Sequence,
        qry_seq: Sequence,
        penalties: &Penalties,
        spare_penalty: usize,
    ) -> Self {
        Self::new_aligned(ref_seq, qry_seq, penalties, spare_penalty, &consecutive_match_reverse)
    }
    fn new_aligned(
        ref_seq: Sequence,
        qry_seq: Sequence,
        penalties: &Penalties,
        spare_penalty: usize,
        match_counter: MatchCounter,
    ) -> Self {
        let ref_len = ref_seq.len();
        let qry_len = qry_seq.len();

        let mut dropoff_wave_front = Self::new_allocated(penalties, spare_penalty);

        let first_match_count = match_counter(ref_seq, qry_seq, 0, 0);

        dropoff_wave_front.wave_front_scores[0].add_first_components(first_match_count);
        
        if first_match_count as usize >= ref_len || first_match_count as usize >= qry_len {
            dropoff_wave_front.update_if_aligned_to_end(0, 0);
            return dropoff_wave_front;
        }

        let optional_last_point = dropoff_wave_front.fill_and_exist_with_last_score_and_k(ref_seq, qry_seq, ref_len, qry_len, spare_penalty, penalties, match_counter);

        if let Some((last_score, last_k)) = optional_last_point {
            dropoff_wave_front.update_if_aligned_to_end(last_score, last_k);
            return dropoff_wave_front;
        } else {
            dropoff_wave_front
        }
    }
    fn new_allocated(penalties: &Penalties, spare_penalty: usize) -> Self {
        let wave_front_score_count = spare_penalty + 1;
        let gap_open_penalty = penalties.o;
        let gap_extend_penalty = penalties.e;

        let mut wave_front_scores: Vec<WaveFrontScore<C>> = Vec::with_capacity(wave_front_score_count);

        let first_wave_front_score = WaveFrontScore::with_max_k(0);

        let optional_penalty_from_one_gap = spare_penalty.checked_sub(gap_open_penalty + gap_extend_penalty);

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
                (0..spare_penalty+1).for_each(|_| {
                    wave_front_scores.push(first_wave_front_score.clone());
                });
            },
        }

        Self {
            last_score: spare_penalty,
            last_k: None,
            wave_front_scores,
        }
    }
    fn fill_and_exist_with_last_score_and_k(
        &mut self,
        ref_seq: Sequence,
        qry_seq: Sequence,
        ref_len: usize,
        qry_len: usize,
        spare_penalty: usize,
        penalties: &Penalties,
        match_counter: MatchCounter,
    ) -> Option<(usize, i32)> {
        for score in 1..=spare_penalty {
            let (mut components_of_score, range_of_k) = C::new_components_and_k_range_of_score(&self, score, penalties);

            let wave_front_score = &mut self.wave_front_scores[score];
    
            for ([m_component, _, _], k) in components_of_score.iter_mut().zip(range_of_k.into_iter()) {
                if m_component.bt() != EMPTY {
                    // Extend & update
                    let mut v = (m_component.fr() - k) as usize;
                    let mut h = m_component.fr() as usize;
                    let match_count = match_counter(ref_seq, qry_seq, v, h);
                    m_component.add_match_count_to_fr(match_count);
                    // Check exit condition
                    v += match_count as usize;
                    h += match_count as usize;
                    if h >= ref_len || v >= qry_len {
                        wave_front_score.update(components_of_score);
                        return Some((score, k));
                    }
                };
            };
            wave_front_score.update(components_of_score);
        }
        None
    }
    fn update_if_aligned_to_end(&mut self, last_score: usize, last_k: i32) {
        self.wave_front_scores.truncate(last_score + 1);
        self.last_score = last_score;
        self.last_k = Some(last_k);
    }
}

#[derive(Debug, Clone)]
pub struct WaveFrontScore<C: Component> {
    pub max_k: i32,
    pub components: Components<C>,
}
impl<C: Component> WaveFrontScore<C> {
    fn with_max_k(max_k: i32) -> Self {
        Self {
            max_k,
            components: Vec::new(),
        }
    }
    fn add_first_components(&mut self, first_match: i32) {
        self.components = vec![[
            C::start_point(first_match),
            C::empty(),
            C::empty(),
        ]];
    }
    pub fn range_of_k(&self) -> Vec<i32> {
        (-self.max_k..=self.max_k).collect()
    }
    fn update(&mut self, new_components: Components<C>) {
        self.components = new_components;
    }
    pub fn component_of_k(&self, k: i32, component_type: usize) -> &C {
        &self.components[(self.max_k + k) as usize][component_type]
    }
    pub fn component_of_k_checked(&self, k: i32, component_type: usize) -> Option<&C> {
        match self.components.get((self.max_k + k) as usize) {
            Some(components) => {
                Some(&components[component_type])
            },
            None => None,
        }
    }
}

// Component Index
pub const M_COMPONENT: usize = 0;
pub const I_COMPONENT: usize = 1;
pub const D_COMPONENT: usize = 2;

pub type Components<C: Component> = Vec<[C; 3]>;

// Backtrace marker
pub const EMPTY: u8 = 0;
pub const FROM_M: u8 = 1;
pub const FROM_I: u8 = 2;
pub const FROM_D: u8 = 3;
pub const START: u8 = 4;

pub trait Component: Sized + Clone {
    fn empty() -> Self;
    fn start_point(first_fr: i32) -> Self;
    fn fr(&self) -> i32;
    fn bt(&self) -> u8;
    fn add_match_count_to_fr(&mut self, match_count: i32);
    fn new_components_and_k_range_of_score(
        dropoff_wave_front: &DropoffWaveFront<Self>, score: usize, penalties: &Penalties,
    ) -> (Components<Self>, Vec<i32>);
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
