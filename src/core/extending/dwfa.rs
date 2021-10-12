// Dropoff Wave Front Algorithm
use super::Penalties;
use super::Sequence;

type MatchCounter<'a> = &'a dyn Fn(Sequence, Sequence, usize, usize) -> i32;

#[derive(Debug)]
pub struct DropoffWaveFront {
    pub last_score: usize,
    pub last_k: Option<i32>,
    pub wave_front_scores: Vec<WaveFrontScore>,
}
impl DropoffWaveFront {
    pub fn new_with_align_forward(
        ref_seq: Sequence,
        qry_seq: Sequence,
        penalties: &Penalties,
        spare_penalty: usize,
    ) -> Self {
        Self::new_with_align(ref_seq, qry_seq, penalties, spare_penalty, &consecutive_match_forward)
    }
    pub fn new_with_align_reverse(
        ref_seq: Sequence,
        qry_seq: Sequence,
        penalties: &Penalties,
        spare_penalty: usize,
    ) -> Self {
        Self::new_with_align(ref_seq, qry_seq, penalties, spare_penalty, &consecutive_match_reverse)
    }
    fn new_with_align(
        ref_seq: Sequence,
        qry_seq: Sequence,
        penalties: &Penalties,
        spare_penalty: usize,
        match_counter: MatchCounter,
    ) -> Self {
        let ref_len = ref_seq.len();
        let qry_len = qry_seq.len();

        let mut dropoff_wave_front = Self::allocated_empty(penalties, spare_penalty);

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
    fn allocated_empty(penalties: &Penalties, spare_penalty: usize) -> Self {
        let wave_front_score_count = spare_penalty + 1;
        let gap_open_penalty = penalties.o;
        let gap_extend_penalty = penalties.e;

        let mut wave_front_scores: Vec<WaveFrontScore> = Vec::with_capacity(wave_front_score_count);

        let first_wave_front_score = WaveFrontScore::with_max_k(0);

        if spare_penalty >= gap_open_penalty + gap_extend_penalty {
            (0..gap_open_penalty + gap_extend_penalty).for_each(|_| {
                wave_front_scores.push(first_wave_front_score.clone());
            });

            let quot = ((spare_penalty - gap_open_penalty - gap_extend_penalty) / gap_extend_penalty) as i32;
            let rem = (spare_penalty - gap_open_penalty - gap_extend_penalty) % gap_extend_penalty;
            for max_k in 1..quot+1 {
                (0..gap_extend_penalty).for_each(|_| {
                    wave_front_scores.push(WaveFrontScore::with_max_k(max_k));
                });
            };
            (0..rem+1).for_each(|_| {
                wave_front_scores.push(WaveFrontScore::with_max_k(quot+1));
            });
        } else {
            (0..spare_penalty+1).for_each(|_| {
                wave_front_scores.push(first_wave_front_score.clone());
            });
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
            let (mut components_of_score, range_of_k) = self.new_components_and_k_range_of_score(score, penalties);

            let wave_front_score = &mut self.wave_front_scores[score];
    
            for ([m_component, _, _], k) in components_of_score.iter_mut().zip(range_of_k.into_iter()) {
                if m_component.bt != EMPTY {
                    // Extend & update
                    let mut v = (m_component.fr - k) as usize;
                    let mut h = m_component.fr as usize;
                    let match_count = match_counter(ref_seq, qry_seq, v, h);
                    m_component.fr += match_count;
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
    fn new_components_and_k_range_of_score(&self, score: usize, penalties: &Penalties) -> (Components, Vec<i32>) {
        let wave_front_score = &self.wave_front_scores[score];
        let mismatch_penalty = penalties.x;
        let gap_open_penalty = penalties.o;
        let gap_extend_penalty = penalties.e;

        let range_of_k = wave_front_score.range_of_k();

        let mut new_components: Components = vec![[Component::empty(); 3]; range_of_k.len()];
    
        // (1) From score: s-o-e
        if let Some(pre_score) = score.checked_sub(gap_open_penalty + gap_extend_penalty) {
            let pre_wave_front_score = &self.wave_front_scores[pre_score];
            for (index_of_k, k) in range_of_k.iter().enumerate() {
                let new_component_of_k = &mut new_components[index_of_k];
                // 1. Update I from M & M from I
                if let Some(pre_m_component) = pre_wave_front_score.component_of_k_checked(k-1, M_COMPONENT) {
                    if pre_m_component.bt != EMPTY {
                        // Update I
                        new_component_of_k[I_COMPONENT] = Component {
                            fr: pre_m_component.fr + 1,
                            bt: FROM_M,
                        };
                    }
                }
                // 2. Update D from M & M from D
                if let Some(pre_m_component) = pre_wave_front_score.component_of_k_checked(k+1, M_COMPONENT) {
                    if pre_m_component.bt != EMPTY {
                        // Update D
                        new_component_of_k[D_COMPONENT] = Component {
                            fr: pre_m_component.fr,
                            bt: FROM_M,
                        };
                    }
                }
            }
        }
        // (2) From score: s-e
        if let Some(pre_score) = score.checked_sub(gap_extend_penalty) {
            let pre_wave_front_score = &self.wave_front_scores[pre_score];
            range_of_k.iter().enumerate().for_each(|(index_of_k, k)| {
                let new_component_of_k = &mut new_components[index_of_k];
                // 1. Update I from I
                if let Some(pre_i_component) = pre_wave_front_score.component_of_k_checked(k-1, I_COMPONENT) {
                    if pre_i_component.bt != EMPTY {
                        // Update I
                        if new_component_of_k[I_COMPONENT].bt == EMPTY || new_component_of_k[I_COMPONENT].fr < pre_i_component.fr + 1 {
                            new_component_of_k[I_COMPONENT] = Component {
                                fr: pre_i_component.fr + 1,
                                bt: FROM_I,
                            };
                        };
                    }
                }
                // 2. Update D from D
                if let Some(pre_d_component) = pre_wave_front_score.component_of_k_checked(k+1, D_COMPONENT) {
                    if pre_d_component.bt != EMPTY {
                        // Update D
                        if new_component_of_k[D_COMPONENT].bt == EMPTY || new_component_of_k[D_COMPONENT].fr < pre_d_component.fr {
                            new_component_of_k[D_COMPONENT] = Component {
                                fr: pre_d_component.fr,
                                bt: FROM_D,
                            };
                        };
                    }
                }
            });
        }
        // (3) From score: s-x
        if let Some(pre_score) = score.checked_sub(mismatch_penalty) {
            let pre_wave_front_score = &self.wave_front_scores[pre_score];
            range_of_k.iter().enumerate().for_each(|(index_of_k, k)| {
                let component_of_s_k = &mut new_components[index_of_k];
                // 1. Update M from M
                let pre_component_index = (pre_wave_front_score.max_k + k) as usize;
                if let Some([pre_m_component, _, _]) = pre_wave_front_score.components.get(pre_component_index) {
                    // Update M
                    component_of_s_k[M_COMPONENT] = Component {
                        fr: pre_m_component.fr + 1,
                        bt: FROM_M,
                    };
                }
                // 2. Update M from I
                if component_of_s_k[I_COMPONENT].bt != EMPTY {
                    if component_of_s_k[M_COMPONENT].bt == EMPTY || component_of_s_k[I_COMPONENT].fr >= component_of_s_k[M_COMPONENT].fr {
                        component_of_s_k[M_COMPONENT] = Component {
                            fr: component_of_s_k[I_COMPONENT].fr,
                            bt: FROM_I,
                        };
                    };
                }
                // 3. Update M from D
                if component_of_s_k[D_COMPONENT].bt != EMPTY {
                    if component_of_s_k[M_COMPONENT].bt == EMPTY || component_of_s_k[D_COMPONENT].fr >= component_of_s_k[M_COMPONENT].fr {
                        component_of_s_k[M_COMPONENT] = Component {
                            fr: component_of_s_k[D_COMPONENT].fr,
                            bt: FROM_D,
                        };
                    };
                }
            });
        }

        (new_components, range_of_k)
    }
    fn update_if_aligned_to_end(&mut self, last_score: usize, last_k: i32) {
        self.wave_front_scores.truncate(last_score + 1);
        self.last_score = last_score;
        self.last_k = Some(last_k);
    }
}

#[derive(Debug, Clone)]
pub struct WaveFrontScore {
    pub max_k: i32,
    pub components: Components,
}
impl WaveFrontScore {
    fn with_max_k(max_k: i32) -> Self {
        Self {
            max_k,
            components: Vec::new(),
        }
    }
    fn add_first_components(&mut self, first_match: i32) {
        self.components = vec![[
            Component { fr: first_match, bt: START },
            Component { fr: 0, bt: EMPTY } ,
            Component { fr: 0, bt: EMPTY } ,
        ]];
    }
    fn range_of_k(&self) -> Vec<i32> {
        (-self.max_k..=self.max_k).collect()
    }
    fn update(&mut self, new_components: Components) {
        self.components = new_components;
    }
    pub fn component_of_k(&self, k: i32, component_type: usize) -> &Component {
        &self.components[(self.max_k + k) as usize][component_type]
    }
    fn component_of_k_checked(&self, k: i32, component_type: usize) -> Option<&Component> {
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

pub type Components = Vec<[Component; 3]>;

// Backtrace marker
pub const EMPTY: u8 = 0;
pub const FROM_M: u8 = 1;
pub const FROM_I: u8 = 2;
pub const FROM_D: u8 = 3;
pub const START: u8 = 4;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Component {
    pub fr: i32,
    pub bt: u8,
}

impl Component {
    fn empty() -> Self {
        Self { fr: 0 , bt: EMPTY }
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
