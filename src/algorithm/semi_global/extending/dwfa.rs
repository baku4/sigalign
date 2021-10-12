use super::Penalties;
use super::Sequence;
use super::{AlignmentOperation, AlignmentType};
use super::{Extension, OperationsOfExtension, OwnedOperations};
use super::{DropoffWaveFront, WaveFrontScore, Components, Component};
use super::{M_COMPONENT, I_COMPONENT, D_COMPONENT, EMPTY, FROM_M, FROM_I, FROM_D, START};

impl DropoffWaveFront {
    pub fn align_right_for_semi_global(
        ref_seq: Sequence,
        qry_seq: Sequence,
        penalties: &Penalties,
        spare_penalty: usize,
    ) -> Option<Extension> {
        let dropoff_wave_front = Self::new_with_align_forward(ref_seq, qry_seq, penalties, spare_penalty);

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
        let dropoff_wave_front = Self::new_with_align_reverse(ref_seq, qry_seq, penalties, spare_penalty);

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
