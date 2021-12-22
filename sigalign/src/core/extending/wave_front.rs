use super::Penalties;
use super::Sequence;

type MatchCounter<'a> = &'a dyn Fn(Sequence, Sequence, usize, usize) -> i32;



// Wave Front



#[derive(Debug)]
pub struct WaveFront {
    pub max_score: usize,
    pub end_point: Option<EndPoint>,
    pub wave_front_scores: Vec<WaveFrontScore>,
}

#[derive(Debug)]
pub struct EndPoint {
    score: usize,
    k: i32,
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
            end_point: None,
            wave_front_scores,
        }
    }
    fn reset(&mut self) {
        self.end_point = None;
    }
    fn align_to_end_point(
        &mut self,
        ref_seq: Sequence,
        qry_seq: Sequence,
        penalties: &Penalties,
        spare_penalty: usize,
        match_counter: MatchCounter,
    ) {
        let ref_len = ref_seq.len();
        let qry_len = qry_seq.len();

        let first_match_count = match_counter(ref_seq, qry_seq, 0, 0);

        self.wave_front_scores[0].add_first_components(first_match_count);

        if first_match_count as usize >= ref_len || first_match_count as usize >= qry_len {
            let end_point = EndPoint { score: 0, k: 0 };
            self.end_point = Some(end_point);
        } else {
            let optional_end_point = self.fill_wave_front_score_until_end(
                ref_seq,
                qry_seq,
                spare_penalty,
                penalties,
                match_counter
            );
            self.end_point = optional_end_point;
        }
        
        
    }
    fn fill_wave_front_score_until_end(
        &mut self,
        ref_seq: Sequence,
        qry_seq: Sequence,
        spare_penalty: usize,
        penalties: &Penalties,
        match_counter: MatchCounter,
    ) -> Option<EndPoint> {
        for score in 1..=spare_penalty {
            let mut next_wave_front_score = self.next_wave_front_score(score, penalties);

            let optional_last_k = next_wave_front_score.extend_components_until_end(ref_seq, qry_seq, match_counter);

            self.wave_front_scores[score] = next_wave_front_score;

            if let Some(last_k) = optional_last_k {
                let end_point = EndPoint { score: score, k: last_k };
                return Some(end_point);
            }
        }
        None
    }
    fn next_wave_front_score(
        &self,
        score: usize,
        penalties: &Penalties,
    ) -> WaveFrontScore {
        // TODO:
    }
    fn update_end_point(&mut self, end_point: EndPoint) {
        self.end_point = Some(end_point);
    }
}

impl WaveFrontScore {
    // New
    fn with_max_k(max_k: i32) -> Self {
        Self {
            max_k,
            components_by_k: Vec::new(),
        }
    }
    // Set
    fn add_first_components(&mut self, first_match_count: i32) {
        self.components_by_k = vec![Components::new_start_point(first_match_count)];
    }
    fn update(&mut self, components_by_k: Vec<Components>) {
        self.components_by_k = components_by_k;
    }
    fn extend_components_until_end(
        &mut self,
        ref_seq: Sequence,
        qry_seq: Sequence,
        match_counter: MatchCounter,
    ) -> Option<i32> {
        for (components, k) in self.components_by_k.iter_mut().zip(-self.max_k..=self.max_k) {
            let m_component = &mut components.m;

            // Extend & update
            let mut v = (m_component.fr - k) as usize;
            let mut h = m_component.fr as usize;
            let match_count = match_counter(ref_seq, qry_seq, v, h);
            m_component.fr += match_count;
            // Check exit condition
            v += match_count as usize;
            h += match_count as usize;
            if h >= ref_seq.len() || v >= qry_seq.len() {
                return Some(k);
            }
        }
        None
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
}



// Components



#[derive(Debug, Clone)]
pub struct Components {
    m: Component,
    i: Component,
    d: Component,
}

#[derive(Debug, Clone)]
pub struct Component {
    pub fr: i32,
    pub deletion_count: u16,
    pub bt: u8,
}
// Backtrace marker
pub const BT_EMPTY: u8 = 0;
pub const BT_FROM_M: u8 = 1;
pub const BT_FROM_I: u8 = 2;
pub const BT_FROM_D: u8 = 3;
pub const BT_START: u8 = 4;

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
    fn empty() -> Self {
        Self {
            fr: 0,
            deletion_count: 0,
            bt: BT_EMPTY,
        }
    }
    fn start_point(first_fr: i32) -> Self {
        Self {
            fr: first_fr,
            deletion_count: 0,
            bt: BT_START,
        }
    }
}