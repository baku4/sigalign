// Dropoff Wave Front Algorithm
use super::Penalties;
use super::Sequence;
use super::{AlignmentOperation, AlignmentType};
use super::{Extension, OperationsOfExtension, OwnedOperations};

type MatchCounter<'a> = &'a dyn Fn(Sequence, Sequence, usize, usize) -> i32;

#[derive(Debug)]
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
    ) -> Option<Extension> {
        let dropoff_wave_front = Self::new_with_align(ref_seq, qry_seq, penalties, spare_penalty, &consecutive_match_forward);

        if dropoff_wave_front.is_extended_to_end() {
            Some(
                dropoff_wave_front.backtrace_from_last_k(penalties)
            )
        } else {
            None
        }
    }
    pub fn align_left_for_semi_global(
        ref_seq: Sequence,
        qry_seq: Sequence,
        penalties: &Penalties,
        spare_penalty: usize,
    ) -> Option<Extension> {
        let dropoff_wave_front = Self::new_with_align(ref_seq, qry_seq, penalties, spare_penalty, &consecutive_match_reverse);

        if dropoff_wave_front.is_extended_to_end() {
            Some(
                dropoff_wave_front.backtrace_from_last_k(penalties)
            )
        } else {
            None
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
    fn is_extended_to_end(&self) -> bool {
        match self.last_k {
            Some(_) => true,
            None => false,
        }
    }
    fn backtrace_from_last_k(
        &self,
        penalties: &Penalties,
    ) -> Extension {
        self.backtrace_from_point(
            self.last_score,
            self.last_k.unwrap(),
            penalties,
        )
    }
    fn backtrace_from_point(
        &self,
        mut score: usize,
        mut k: i32,
        penalties: &Penalties,
    ) -> Extension {
        let wave_front_scores = &self.wave_front_scores;
        let mut operation_length: usize = 0;
        let mut operations: Vec<AlignmentOperation> = Vec::new(); // TODO: Capacity can be applied?
        
        let mut wave_front_score: &WaveFrontScore = &wave_front_scores[score];
        let mut component_type: usize = M_COMPONENT;
        let mut component: &Component = wave_front_score.component_of_k(k, component_type);
        let mut fr: i32 = component.fr;
        
        loop {
            match component_type {
                /* M */
                M_COMPONENT => {
                    match component.bt {
                        FROM_M => {
                            // (1) Next score
                            score -= penalties.x;
                            // (2) Next k
                            // not change
                            // (3) Next WFS
                            wave_front_score = &wave_front_scores[score];
                            // (4) Component type
                            // not change
                            // (5) Next component
                            component = wave_front_score.component_of_k(k, M_COMPONENT);
                            // (6) Next fr
                            let next_fr = component.fr;
                            // (7) Add operation
                            let match_count = (fr - next_fr - 1) as u32;
                            if match_count == 0 {
                                if let Some(
                                    AlignmentOperation {
                                        alignment_type: AlignmentType::Subst,
                                        count: last_fr
                                    }) = operations.last_mut() {
                                    *last_fr += 1;
                                } else {
                                    operations.push(
                                        AlignmentOperation {
                                            alignment_type: AlignmentType::Subst,
                                            count: 1
                                        }
                                    );
                                }
                            } else {
                                operations.push(
                                    AlignmentOperation {
                                        alignment_type: AlignmentType::Match,
                                        count: match_count
                                    }
                                );
                                operations.push(
                                    AlignmentOperation {
                                        alignment_type: AlignmentType::Subst,
                                        count: 1
                                    }
                                );
                            }
                            operation_length += (match_count + 1) as usize;
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
                            component = wave_front_score.component_of_k(k, I_COMPONENT);
                            // (6) Next fr
                            let next_fr = component.fr;
                            // (7) Add Cigar
                            let match_count = (fr-next_fr) as u32;
                            if match_count != 0 {
                                operations.push(
                                    AlignmentOperation {
                                        alignment_type: AlignmentType::Match,
                                        count: match_count
                                    }
                                );
                            }

                            operation_length += match_count as usize;
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
                            component = wave_front_score.component_of_k(k, D_COMPONENT);
                            // (6) Next fr
                            let next_fr = component.fr;
                            // (7) Add Cigar
                            let match_count = (fr-next_fr) as u32;
                            if match_count != 0 {
                                operations.push(
                                    AlignmentOperation {
                                        alignment_type: AlignmentType::Match,
                                        count: match_count
                                    }
                                );
                            }
                            operation_length += match_count as usize;
                            // (9) Next fr to fr
                            fr = next_fr;
                        },
                        _ => { // START_POINT
                            if fr != 0 {
                                operations.push(
                                    AlignmentOperation {
                                        alignment_type: AlignmentType::Match,
                                        count: fr as u32,
                                    }
                                );
                            };
                            operation_length += fr as usize;
                            // shrink
                            operations.shrink_to_fit();
                            // extension of current anchor
                            let extension = Extension {
                                penalty: score,
                                length: operation_length,
                                operations: OperationsOfExtension::Own(
                                    OwnedOperations {
                                        operations: operations,
                                    }
                                )
                            };
                            return extension;
                        }
                    }
                },
                /* I */
                I_COMPONENT => {
                    match component.bt {
                        FROM_M => {
                            // (1) Next score
                            score -= penalties.o + penalties.e;
                            // (2) Next k
                            k -= 1;
                            // (3) Next WFS
                            wave_front_score = &wave_front_scores[score];
                            // (4) Component type
                            component_type = 0;
                            // (5) Next component
                            component = wave_front_score.component_of_k(k, M_COMPONENT);
                            // (6) Next fr
                            let next_fr = component.fr;
                            // (7) Add operation
                            if let Some(
                                AlignmentOperation {
                                    alignment_type: AlignmentType::Insertion,
                                    count: last_fr
                                }) = operations.last_mut() {
                                *last_fr += 1;
                            } else {
                                operations.push(
                                    AlignmentOperation {
                                        alignment_type: AlignmentType::Insertion,
                                        count: 1,
                                    }
                                )
                            }
                            operation_length += 1;
                            // (9) Next fr to fr
                            fr = next_fr;
                        },
                        _ => { // FROM_I
                            // (1) Next score
                            score -= penalties.e;
                            // (2) Next k
                            k -= 1;
                            // (3) Next WFS
                            wave_front_score = &wave_front_scores[score];
                            // (4) Component type
                            // not change
                            // (5) Next component
                            component = wave_front_score.component_of_k(k, I_COMPONENT);
                            // (6) Next fr
                            let next_fr = component.fr;
                            // (7) Add operation
                            if let Some(
                                AlignmentOperation {
                                    alignment_type: AlignmentType::Insertion,
                                    count: last_fr
                                }) = operations.last_mut() {
                                *last_fr += 1;
                            } else {
                                operations.push(
                                    AlignmentOperation {
                                        alignment_type: AlignmentType::Insertion,
                                        count: 1,
                                    }
                                )
                            }
                            operation_length += 1;
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
                            wave_front_score = &wave_front_scores[score];
                            // (4) Component type
                            component_type = 0;
                            // (5) Next component
                            component = wave_front_score.component_of_k(k, M_COMPONENT);
                            // (6) Next fr
                            let next_fr = component.fr;
                            // (7) Add operation
                            if let Some(
                                AlignmentOperation {
                                    alignment_type: AlignmentType::Deletion,
                                    count: last_fr
                                }) = operations.last_mut() {
                                *last_fr += 1;
                            } else {
                                operations.push(
                                    AlignmentOperation {
                                        alignment_type: AlignmentType::Deletion,
                                        count: 1,
                                    }
                                )
                            }
                            operation_length += 1;
                            // (9) Next fr to fr
                            fr = next_fr;
                        },
                        _ => { // FROM_D
                            // (1) Next score
                            score -= penalties.e;
                            // (2) Next k
                            k += 1;
                            // (3) Next WFS
                            wave_front_score = &wave_front_scores[score];
                            // (4) Component type
                            // not change
                            // (5) Next component
                            component = wave_front_score.component_of_k(k, D_COMPONENT);
                            // (6) Next fr
                            let next_fr = component.fr;
                            // (7) Add operation
                            if let Some(
                                AlignmentOperation {
                                    alignment_type: AlignmentType::Deletion,
                                    count: last_fr
                                }) = operations.last_mut() {
                                *last_fr += 1;
                            } else {
                                operations.push(
                                    AlignmentOperation {
                                        alignment_type: AlignmentType::Deletion,
                                        count: 1,
                                    }
                                )
                            }
                            operation_length += 1;
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
    fn component_of_k(&self, k: i32, component_type: usize) -> &Component {
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
const M_COMPONENT: usize = 0;
const I_COMPONENT: usize = 1;
const D_COMPONENT: usize = 2;

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
