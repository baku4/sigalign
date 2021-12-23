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
        let penalty_from_start_point = score;

        let wave_front_scores = &self.wave_front_scores;
        let mut operations: Vec<AlignmentOperation> = Vec::new(); // TODO: Capacity can be applied?
        
        let mut wave_front_score = &wave_front_scores[score];

        // Init
        let mut component_type = ComponentType::M;
        let mut component = &wave_front_score.components_by_k[index_of_component].m;

        let mut k = -wave_front_score.max_k + index_of_component as i32;
        let mut fr = component.fr;

        let operation_length = fr as usize + component.deletion_count as usize;
        let deletion_count: u32 = component.deletion_count as u32;
        let insertion_count: u32 = deletion_count + k as u32;
        // FIXME: length can be calculated directly from deletion count and fr.
        
        loop {
            match component_type {
                /* M */
                ComponentType::M => {
                    match component.bt {
                        BackTraceMarker::FromM => {
                            // (1) Next score
                            score -= penalties.x;
                            // (2) Next k
                            // not change
                            // (3) Next WFS
                            wave_front_score = &wave_front_scores[score];
                            // (4) Component type
                            // not change
                            // (5) Next component
                            component = wave_front_score.m_component_of_k(k);
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
                        BackTraceMarker::FromI => {
                            // (1) Next score
                            // not change
                            // (2) Next k
                            // not change
                            // (3) Next WFS
                            // not change
                            // (4) Component type
                            component_type = ComponentType::I;
                            // (5) Next component
                            component = wave_front_score.i_component_of_k(k);
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
                        BackTraceMarker::FromD => {
                            // (1) Next score
                            // not change
                            // (2) Next k
                            // not change
                            // (3) Next WFS
                            // not change
                            // (4) Component type
                            component_type = ComponentType::D;
                            // (5) Next component
                            component = wave_front_score.d_component_of_k(k);
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
                                penalty: penalty_from_start_point,
                                length: operation_length,
                                insertion_count,
                                deletion_count,
                                operations: operations,
                            };
                            return extension;
                        }
                    }
                },
                /* I */
                ComponentType::I => {
                    match component.bt {
                        FROM_M => {
                            // (1) Next score
                            score -= penalties.o + penalties.e;
                            // (2) Next k
                            k -= 1;
                            // (3) Next WFS
                            wave_front_score = &wave_front_scores[score];
                            // (4) Component type
                            component_type = ComponentType::M;
                            // (5) Next component
                            component = wave_front_score.m_component_of_k(k);
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
                            component = wave_front_score.i_component_of_k(k);
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
                ComponentType::D => {
                    match component.bt {
                        FROM_M => {
                            // (1) Next score
                            score -= penalties.o + penalties.e;
                            // (2) Next k
                            k += 1;
                            // (3) Next WFS
                            wave_front_score = &wave_front_scores[score];
                            // (4) Component type
                            component_type = ComponentType::M;
                            // (5) Next component
                            component = wave_front_score.m_component_of_k(k);
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
                            component = wave_front_score.d_component_of_k(k);
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

enum ComponentType {
    M,
    I,
    D,
}