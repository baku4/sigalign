use super::Penalties;
use super::Sequence;
use super::{AlignmentOperation, AlignmentType};
use super::{Extension, OperationsOfExtension, OwnedOperations};
use super::{DropoffWaveFront, WaveFrontScore, Components, Component};
use super::{M_COMPONENT, I_COMPONENT, D_COMPONENT, EMPTY, FROM_M, FROM_I, FROM_D, START};

impl DropoffWaveFront<ComponentSemiGlobal> {
    pub fn align_right_for_semi_global(
        ref_seq: Sequence,
        qry_seq: Sequence,
        penalties: &Penalties,
        spare_penalty: usize,
    ) -> Option<Extension> {
        let dropoff_wave_front = Self::aligned_forward(ref_seq, qry_seq, penalties, spare_penalty);

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
        let dropoff_wave_front = Self::aligned_reverse(ref_seq, qry_seq, penalties, spare_penalty);

        if dropoff_wave_front.is_extended_to_end() {
            Some(
                dropoff_wave_front.backtrace_from_last_k(penalties)
            )
        } else {
            None
        }
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
        let mut insertion_count: u32 = 0;
        let mut deletion_count: u32 = 0;
        let mut operations: Vec<AlignmentOperation> = Vec::new(); // TODO: Capacity can be applied?
        
        let mut wave_front_score: &WaveFrontScore<ComponentSemiGlobal> = &wave_front_scores[score];
        let mut component_type: usize = M_COMPONENT;
        let mut component: &ComponentSemiGlobal = wave_front_score.component_of_k(k, component_type);
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
                                penalty: self.last_score,
                                length: operation_length,
                                insertion_count,
                                deletion_count,
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
                            insertion_count += 1;
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
                            insertion_count += 1;
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
                            deletion_count += 1;
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
                            deletion_count += 1;
                            // (9) Next fr to fr
                            fr = next_fr;
                        },
                    }
                },
            };
        }
    }
}


#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ComponentSemiGlobal {
    pub fr: i32,
    pub bt: u8,
}

impl Component for ComponentSemiGlobal {
    fn empty() -> Self {
        Self { fr: 0 , bt: EMPTY }
    }
    fn start_point(first_fr: i32) -> Self {
        Self { fr: first_fr, bt: START }
    }
    fn fr(&self) -> i32 {
        self.fr
    }
    fn bt(&self) -> u8 {
        self.bt
    }
    fn add_match_count_to_fr(&mut self, match_count: i32) {
        self.fr += match_count;
    }
    fn new_components_and_k_range_of_score(
        dropoff_wave_front: &DropoffWaveFront<Self>, score: usize, penalties: &Penalties,
    ) -> (Components<Self>, Vec<i32>) {
        let wave_front_score = &dropoff_wave_front.wave_front_scores[score];
        let mismatch_penalty = penalties.x;
        let gap_open_penalty = penalties.o;
        let gap_extend_penalty = penalties.e;

        let range_of_k = wave_front_score.range_of_k();

        let mut new_components: Components<Self> = vec![[Self::empty(); 3]; range_of_k.len()];
    
        // (1) From score: s-o-e
        if let Some(pre_score) = score.checked_sub(gap_open_penalty + gap_extend_penalty) {
            let pre_wave_front_score = &dropoff_wave_front.wave_front_scores[pre_score];
            for (index_of_k, k) in range_of_k.iter().enumerate() {
                let new_component_of_k = &mut new_components[index_of_k];
                // 1. Update I from M & M from I
                if let Some(pre_m_component) = pre_wave_front_score.component_of_k_checked(k-1, M_COMPONENT) {
                    if pre_m_component.bt != EMPTY {
                        // Update I
                        new_component_of_k[I_COMPONENT] = Self {
                            fr: pre_m_component.fr + 1,
                            bt: FROM_M,
                        };
                    }
                }
                // 2. Update D from M & M from D
                if let Some(pre_m_component) = pre_wave_front_score.component_of_k_checked(k+1, M_COMPONENT) {
                    if pre_m_component.bt != EMPTY {
                        // Update D
                        new_component_of_k[D_COMPONENT] = Self {
                            fr: pre_m_component.fr,
                            bt: FROM_M,
                        };
                    }
                }
            }
        }
        // (2) From score: s-e
        if let Some(pre_score) = score.checked_sub(gap_extend_penalty) {
            let pre_wave_front_score = &dropoff_wave_front.wave_front_scores[pre_score];
            range_of_k.iter().enumerate().for_each(|(index_of_k, k)| {
                let new_component_of_k = &mut new_components[index_of_k];
                // 1. Update I from I
                if let Some(pre_i_component) = pre_wave_front_score.component_of_k_checked(k-1, I_COMPONENT) {
                    if pre_i_component.bt != EMPTY {
                        // Update I
                        if new_component_of_k[I_COMPONENT].bt == EMPTY || new_component_of_k[I_COMPONENT].fr < pre_i_component.fr + 1 {
                            new_component_of_k[I_COMPONENT] = Self {
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
                            new_component_of_k[D_COMPONENT] = Self {
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
            let pre_wave_front_score = &dropoff_wave_front.wave_front_scores[pre_score];
            range_of_k.iter().enumerate().for_each(|(index_of_k, k)| {
                let component_of_s_k = &mut new_components[index_of_k];
                // 1. Update M from M
                let pre_component_index = (pre_wave_front_score.max_k + k) as usize;
                if let Some([pre_m_component, _, _]) = pre_wave_front_score.components.get(pre_component_index) {
                    // Update M
                    component_of_s_k[M_COMPONENT] = Self {
                        fr: pre_m_component.fr + 1,
                        bt: FROM_M,
                    };
                }
                // 2. Update M from I
                if component_of_s_k[I_COMPONENT].bt != EMPTY {
                    if component_of_s_k[M_COMPONENT].bt == EMPTY || component_of_s_k[I_COMPONENT].fr >= component_of_s_k[M_COMPONENT].fr {
                        component_of_s_k[M_COMPONENT] = Self {
                            fr: component_of_s_k[I_COMPONENT].fr,
                            bt: FROM_I,
                        };
                    };
                }
                // 3. Update M from D
                if component_of_s_k[D_COMPONENT].bt != EMPTY {
                    if component_of_s_k[M_COMPONENT].bt == EMPTY || component_of_s_k[D_COMPONENT].fr >= component_of_s_k[M_COMPONENT].fr {
                        component_of_s_k[M_COMPONENT] = Self {
                            fr: component_of_s_k[D_COMPONENT].fr,
                            bt: FROM_D,
                        };
                    };
                }
            });
        }

        (new_components, range_of_k)
    }
}