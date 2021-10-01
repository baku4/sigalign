// Dropoff Wave Front Algorithm
use crate::{Result, error_msg};
use super::Penalties;
use super::Sequence;
use super::{AlignmentOperation, AlignmentType};
use super::{OwnedOperations, PointerToOperations};

type MatchCounter<'a> = &'a dyn Fn(Sequence, Sequence, usize, usize) -> i32;

use std::collections::HashMap;

pub struct DropoffWaveFront {
    last_score: usize,
    last_k: Option<i32>,
    wave_front_scores: Vec<WaveFrontScore>,
}
impl DropoffWaveFront {
    pub fn align_right_for_semi_global(
        ref_seq: Sequence,
        qry_seq: Sequence,
        penalties: &Penalties,
        spare_penalty: usize,
        position_gaps_of_checkpoints: HashMap<usize, (usize, usize)>,
    ) {
        let dropoff_wave_front = Self::new_with_align(ref_seq, qry_seq, penalties, spare_penalty, &consecutive_match_forward);

        if dropoff_wave_front.is_extended_to_end() {
            let owned_operations = dropoff_wave_front.backtrace_from_last_k(); //TODO: NEXT
        }
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
            dropoff_wave_front.update_if_aligned_to_end(0);
            return dropoff_wave_front;
        }

        for score in 1..=spare_penalty {
            let optional_last_k = dropoff_wave_front.fill_wave_front_score_and_exist_with_last_k(ref_seq, qry_seq, ref_len, qry_len, score, penalties, match_counter);

            if let Some(last_k) = optional_last_k {
                dropoff_wave_front.update_if_aligned_to_end(last_k);
                return dropoff_wave_front;
            }
        }

        dropoff_wave_front
    }
    fn allocated_empty(penalties: &Penalties, spare_penalty: usize) -> Self {
        let wave_front_score_count = spare_penalty + 1;
        let gap_open_penalty = penalties.o;
        let gap_extend_penalty = penalties.e;

        let mut wave_front_scores: Vec<WaveFrontScore> = Vec::with_capacity(wave_front_score_count);

        let first_wave_front_score = WaveFrontScore::with_max_k(0);
        (0..gap_open_penalty + gap_extend_penalty).for_each(|_| {
            wave_front_scores.push(first_wave_front_score.clone());
        });

        if spare_penalty >= gap_open_penalty + gap_extend_penalty {
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
        }

        Self {
            last_score: spare_penalty,
            last_k: None,
            wave_front_scores,
        }
    }
    fn fill_wave_front_score_and_exist_with_last_k(
        &mut self,
        ref_seq: Sequence,
        qry_seq: Sequence,
        ref_len: usize,
        qry_len: usize,
        score: usize,
        penalties: &Penalties,
        match_counter: MatchCounter,
    ) -> Option<i32> {
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
                    return Some(k);
                }
            };
        };
        wave_front_score.update(components_of_score);
        None
    }
    fn new_components_and_k_range_of_score(&self, score: usize, penalties: &Penalties) -> (Components, Vec<i32>) {
        let wave_front_score = &self.wave_front_scores[score];
        let mismatch_penalty = penalties.x;
        let gap_open_penalty = penalties.o;
        let gap_extend_penalty = penalties.e;

        let range_of_k = wave_front_score.range_of_k();

        let mut components: Components = vec![[Component::empty(); 3]; range_of_k.len()];
    
        // (1) From score: s-o-e
        if let Some(pre_score) = score.checked_sub(gap_open_penalty + gap_extend_penalty) {
            let max_k_of_pre_score = self.wave_front_scores[pre_score].max_k;
            let pre_wave_front_score = &self.wave_front_scores[pre_score];
            for (index_of_k, k) in range_of_k.iter().enumerate() {
                let component_of_k = &mut components[index_of_k];
                // 1. Update I from M & M from I
                let mut component_index = max_k_of_pre_score + k - 1;
                if let Some([pre_m, _, _]) = pre_wave_front_score.components.get(component_index as usize) {
                    if pre_m.bt != EMPTY {
                        // Update I
                        component_of_k[1] = Component {
                            fr: pre_m.fr + 1,
                            bt: FROM_M,
                        };
                        
                    }
                }
                // 2. Update D from M & M from D
                component_index += 2;
                if let Some([pre_m, _, _]) = pre_wave_front_score.components.get(component_index as usize) {
                    if pre_m.bt != EMPTY {
                        // Update D
                        component_of_k[2] = Component {
                            fr: pre_m.fr,
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
                let component_of_k = &mut components[index_of_k];
                // 1. Update I from I
                let mut component_index = pre_wave_front_score.max_k + k - 1;
                if let Some([_, pre_i, _]) = pre_wave_front_score.components.get(component_index as usize) {
                    if pre_i.bt != EMPTY {
                        // Update I
                        if component_of_k[1].bt == EMPTY || component_of_k[1].fr > pre_i.fr + 1 {
                            component_of_k[1] = Component {
                                fr: pre_i.fr + 1,
                                bt: FROM_I,
                            };
                        };
                    }
                }
                // 2. Update D from D
                component_index += 2;
                if let Some([_, _, pre_d]) = pre_wave_front_score.components.get(component_index as usize) {
                    if pre_d.bt != EMPTY {
                        // Update D
                        if component_of_k[2].bt == EMPTY || component_of_k[2].fr > pre_d.fr {
                            component_of_k[2] = Component {
                                fr: pre_d.fr,
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
                let component_of_k = &mut components[index_of_k];
                // 1. Update M from M
                let component_index = pre_wave_front_score.max_k + k;
                if let Some([pre_m, _, _]) = pre_wave_front_score.components.get(component_index as usize) {
                    // Update M
                    component_of_k[0] = Component {
                        fr: pre_m.fr + 1,
                        bt: FROM_M,
                    };
                }
                // 2. Update M from I
                if component_of_k[1].bt != EMPTY {
                    if component_of_k[0].bt == EMPTY || component_of_k[1].fr >= component_of_k[0].fr {
                        component_of_k[0] = Component {
                            fr: component_of_k[1].fr,
                            bt: FROM_I,
                        };
                    };
                }
                // 3. Update M from D
                if component_of_k[2].bt != EMPTY {
                    if component_of_k[0].bt == EMPTY || component_of_k[2].fr >= component_of_k[0].fr {
                        component_of_k[0] = Component {
                            fr: component_of_k[2].fr,
                            bt: FROM_D,
                        };
                    };
                }
            });
        }

        (components, range_of_k)
    }
    fn update_if_aligned_to_end(&mut self, last_k: i32) {
        let last_score = self.wave_front_scores.len() + 1;
        self.wave_front_scores.truncate(last_score);
        self.last_score = last_score;
        self.last_k = Some(last_k);
    }
    fn is_extended_to_end(&self) -> bool {
        match self.last_k {
            Some(_) => true,
            None => false,
        }
    }
    fn backtrace_from_last_k(&self) {

    }
    fn bactrace_from_point(&self, score: usize, k: i32) {
        let mut operation_length: usize = 0;
        let mut to_check_index: HashSet<usize> = HashSet::from_iter(0..check_points_values.len());
        // FIXME: check if this cap is enough.
        let mut cigar: Operations = Vec::with_capacity(score);
        let mut checkpoint_backtrace: RefToBacktrace = HashMap::with_capacity(check_points_values.len());
        
        // FIRST COMP
        let mut wfs: &WaveFrontScore = &wf[score];
        let mut component_type: usize = 0;
        let mut component: &Component = wfs.comp_with_k(k, 0);
        let mut fr: WfFrPoint = component.fr;

        // BACKTRACE
        loop {
            match component_type {
                /* M */
                0 => {
                    match component.bt {
                        FROM_M => {
                            // (1) Next score
                            score -= penalties.x;
                            // (2) Next k
                            // not change
                            // (3) Next WFS
                            wfs = &wf[score];
                            // (4) Component type
                            // not chnage
                            // (5) Next component
                            component = wfs.comp_with_k(k, 0);
                            // (6) Next fr
                            let next_fr = component.fr;
                            // (7) Add Cigar
                            let match_count = (fr - next_fr - 1) as OperationLength;
                            if match_count == 0 {
                                if let Some((Opr::Subst, last_fr)) = cigar.last_mut() {
                                    *last_fr += 1;
                                } else {
                                    cigar.push((Opr::Subst, 1));
                                }
                            } else {
                                cigar.push((Opr::Match, match_count));
                                cigar.push((Opr::Subst, 1));
                            }
                            operation_length += (match_count + 1) as usize;
                            // (8) Check if anchor is passed
                            for checkpoint_index in to_check_index.clone() {
                                let &(anchor_index, size, checkpoint_k, checkpoint_fr) = &check_points_values[checkpoint_index];
                                if (checkpoint_k == k) && (checkpoint_fr <= fr) && (checkpoint_fr - size >= next_fr) {
                                    checkpoint_backtrace.insert(
                                        anchor_index,
                                        (
                                            score + penalties.x,
                                            operation_length - (checkpoint_fr - next_fr) as usize,
                                        ),
                                    );
                                    to_check_index.remove(&checkpoint_index);
                                }
                            }
                            // (9) Next fr to fr
                            fr = next_fr;
                        },
                        FROM_I => {
                            // (1) Next score
                            // not change
                            // (2) Next k
                            // not change
                            // (3) Next WFS
                            // not change
                            // (4) Component type
                            component_type = 1;
                            // (5) Next component
                            component = wfs.comp_with_k(k, 1);
                            // (6) Next fr
                            let next_fr = component.fr;
                            // (7) Add Cigar
                            let match_count = (fr-next_fr) as OperationLength;
                            if match_count != 0 {
                                cigar.push((Opr::Match, match_count));
                            }
                            operation_length += match_count as usize;
                            // (8) Check if anchor is passed
                            for checkpoint_index in to_check_index.clone() {
                                let &(anchor_index, size, checkpoint_k, checkpoint_fr) = &check_points_values[checkpoint_index];
                                if (checkpoint_k == k) && (checkpoint_fr <= fr) && (checkpoint_fr - size >= next_fr) {
                                    checkpoint_backtrace.insert(
                                        anchor_index,
                                        (
                                            score,
                                            operation_length - (checkpoint_fr - next_fr) as usize,
                                        ),
                                    );
                                    to_check_index.remove(&checkpoint_index);
                                }
                            }
                            // (9) Next fr to fr
                            fr = next_fr;
                        },
                        FROM_D => {
                            // (1) Next score
                            // not change
                            // (2) Next k
                            // not change
                            // (3) Next WFS
                            // not change
                            // (4) Component type
                            component_type = 2;
                            // (5) Next component
                            component = wfs.comp_with_k(k, 2);
                            // (6) Next fr
                            let next_fr = component.fr;
                            // (7) Add Cigar
                            let match_count = (fr-next_fr) as OperationLength;
                            if match_count != 0 {
                                cigar.push((Opr::Match, match_count));
                            }
                            operation_length += match_count as usize;
                            // (8) Check if anchor is passed
                            for checkpoint_index in to_check_index.clone() {
                                let &(anchor_index, size, checkpoint_k, checkpoint_fr) = &check_points_values[checkpoint_index];
                                if (checkpoint_k == k) && (checkpoint_fr <= fr) && (checkpoint_fr - size >= next_fr) {
                                    checkpoint_backtrace.insert(
                                        anchor_index,
                                        (
                                            score,
                                            operation_length - (checkpoint_fr - next_fr) as usize,
                                        ),
                                    );
                                    to_check_index.remove(&checkpoint_index);
                                }
                            }
                            // (9) Next fr to fr
                            fr = next_fr;
                        },
                        _ => { // START_POINT
                            if fr != 0 {
                                cigar.push((Opr::Match, fr as OperationLength));
                            };
                            operation_length += fr as usize;
                            // shrink
                            cigar.shrink_to_fit();
                            checkpoint_backtrace.shrink_to_fit();
                            return ((cigar, operation_length), checkpoint_backtrace);
                        }
                    }
                },
                /* I */
                1 => {
                    match component.bt {
                        FROM_M => {
                            // (1) Next score
                            score -= penalties.o + penalties.e;
                            // (2) Next k
                            k -= 1;
                            // (3) Next WFS
                            wfs = &wf[score];
                            // (4) Component type
                            component_type = 0;
                            // (5) Next component
                            component = wfs.comp_with_k(k, 0);
                            // (6) Next fr
                            let next_fr = component.fr;
                            // (7) Add Cigar
                            if let Some((Opr::Ins, last_fr)) = cigar.last_mut() {
                                *last_fr += 1;
                            } else {
                                cigar.push((Opr::Ins, 1));
                            }
                            operation_length += 1;
                            // (8) Check if anchor is passed
                            // not needed
                            // (9) Next fr to fr
                            fr = next_fr;
                        },
                        _ => { // FROM_I
                            // (1) Next score
                            score -= penalties.e;
                            // (2) Next k
                            k -= 1;
                            // (3) Next WFS
                            wfs = &wf[score];
                            // (4) Component type
                            // not change
                            // (5) Next component
                            component = wfs.comp_with_k(k, 1);
                            // (6) Next fr
                            let next_fr = component.fr;
                            // (7) Add Cigar
                            if let Some((Opr::Ins, last_fr)) = cigar.last_mut() {
                                *last_fr += 1;
                            } else {
                                cigar.push((Opr::Ins, 1));
                            }
                            operation_length += 1;
                            // (8) Check if anchor is passed
                            // not needed
                            // (9) Next fr to fr
                            fr = next_fr;
                        },
                    }
                },
                /* D */
                _ => {
                    match component.bt {
                        FROM_M => {
                            // (1) Next score
                            score -= penalties.o + penalties.e;
                            // (2) Next k
                            k += 1;
                            // (3) Next WFS
                            wfs = &wf[score];
                            // (4) Component type
                            component_type = 0;
                            // (5) Next component
                            component = wfs.comp_with_k(k, 0);
                            // (6) Next fr
                            let next_fr = component.fr;
                            // (7) Add Cigar
                            if let Some((Opr::Del, last_fr)) = cigar.last_mut() {
                                *last_fr += 1;
                            } else {
                                cigar.push((Opr::Del, 1));
                            }
                            operation_length += 1;
                            // (8) Check if anchor is passed
                            // not needed
                            // (9) Next fr to fr
                            fr = next_fr;
                        },
                        _ => { // FROM_D
                            // (1) Next score
                            score -= penalties.e;
                            // (2) Next k
                            k += 1;
                            // (3) Next WFS
                            wfs = &wf[score];
                            // (4) Component type
                            // not change
                            // (5) Next component
                            component = wfs.comp_with_k(k, 2);
                            // (6) Next fr
                            let next_fr = component.fr;
                            // (7) Add Cigar
                            if let Some((Opr::Del, last_fr)) = cigar.last_mut() {
                                *last_fr += 1;
                            } else {
                                cigar.push((Opr::Del, 1));
                            }
                            operation_length += 1;
                            // (8) Check if anchor is passed
                            // not needed
                            // (9) Next fr to fr
                            fr = next_fr;
                        },
                    }
                },
            };
        }
    }
}

#[derive(Debug, Clone)]
struct WaveFrontScore {
    max_k: i32,
    components: Components,
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
}

type Components = Vec<[Component; 3]>;

// Backtrace marker
const EMPTY: u8 = 0;
const FROM_M: u8 = 1;
const FROM_I: u8 = 2;
const FROM_D: u8 = 3;
const START: u8 = 4;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct Component {
    fr: i32,
    bt: u8,
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