use crate::{SequenceLength, OperationLength, Penalty};
use crate::io::cigar::{Cigar, Operation};
use super::{Penalties, Alignment};

use std::collections::{HashSet, HashMap};
use std::iter::FromIterator;

/* TYPES */
// WaveFront Align
type WfScore = usize;
type WfK = i32;
type WfFrPoint = i32;
// WaveFront Backtrace
type BackTrace = u8;
const EMPTY: u8 = 0;
const FROM_M: u8 = 1;
const FROM_I: u8 = 2;
const FROM_D: u8 = 3;
const START_POINT: u8 = 4;

// TODO: del pub

#[derive(Clone, Default, Debug)]
pub struct WFS {
    m_fr: WfFrPoint,
    i_fr: WfFrPoint,
    d_fr: WfFrPoint,
    m_bt: BackTrace,
    i_bt: BackTrace,
    d_bt: BackTrace,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct Component {
    fr: WfFrPoint,
    bt: BackTrace,
}

impl Default for Component {
    fn default() -> Self {
        Self { fr: 0, bt: 0 }
    }
}

type WfsComps = Vec<[Component; 3]>;

#[derive(Debug, Clone)]
pub struct WaveFrontScore {
    max_k: WfK,
    comp: WfsComps,
}

impl WaveFrontScore {
    #[inline]
    fn with_max_k(max_k: usize) -> Self {
        Self {
            max_k: max_k as WfK,
            // comp: vec![[Component::default(); 3]; 2*max_k+1],
            comp: Vec::new(),
        }
    }
    #[inline]
    fn comp_with_k(&self, k: i32, component_type: usize) -> &Component {
        &self.comp[(self.max_k + k) as usize][component_type]
    }
}

pub type WaveFront = Vec<WaveFrontScore>;

impl Penalties {
    #[inline]
    fn wf_init(&self, score: WfScore) -> WaveFront {
        let mut wf: WaveFront = Vec::with_capacity(score+1);
        
        (0..self.o+self.e).for_each(|_| {
            wf.push(WaveFrontScore::with_max_k(0));
        });
        
        if score >= self.o + self.e {
            let quot = (score-self.o-self.e)/self.e;
            let rem = (score-self.o-self.e)%self.e;
            for max_k in 1..quot+1 {
                (0..self.e).for_each(|_| {
                    wf.push(WaveFrontScore::with_max_k(max_k));
                });
            };
            (0..rem+1).for_each(|_| {
                wf.push(WaveFrontScore::with_max_k(quot+1));
            });
        }
        wf
    }
}

type WfRes = (WaveFront, WfScore, WfK); // (WaveFront, Score, Last k)

#[inline]
pub fn dropout_wf_align(
    qry_seq: &[u8], ref_seq: &[u8],
    penalty_spare: WfScore, penalties: &Penalties,
    forward: bool,
) -> Result<WfRes, WaveFront> {
    let qry_len = qry_seq.len(); // n
    let ref_len = ref_seq.len(); // m

    // INIT
    let mut wf: WaveFront = penalties.wf_init(penalty_spare);
    let fr_offset = seqs_compare(qry_seq, ref_seq, 0, 0, forward);
    let first_wfs_comp: Vec<[Component; 3]> = vec![[
        Component { fr: fr_offset, bt: START_POINT },
        Component::default(),
        Component::default(),
    ]];
    wf[0].comp = first_wfs_comp;
    if fr_offset as usize >= ref_len || fr_offset as usize >= qry_len {
        return Ok((wf,0,0))
    }

    // MAIN
    for score in 1..=penalty_spare {
        let (exist_with_k, wfs_comps) = dropout_wf_next(
            &wf,
            score,
            penalties,
            ref_seq,
            qry_seq,
            ref_len,
            qry_len,
            forward,
        );
        // Update Component
        wf[score].comp = wfs_comps;
        // println!("wf score:{}\n{:#?}", score, wf[0..score+1].iter());
        if let Some(last_k) = exist_with_k {
            // truncate
            wf.truncate(score+1);
            return Ok((wf, score, last_k));
        }
    };
    // Return Err
    Err(wf)
}

#[inline]
fn seqs_compare(qry_seq: &[u8], ref_seq: &[u8], v: usize, h: usize, forward: bool) -> i32 {
    let mut fr_to_add: i32 = 0;
    if forward {
        for (v1, v2) in qry_seq[v..].iter().zip(ref_seq[h..].iter()) {
            if *v1 == *v2 {
                fr_to_add += 1;
            } else {
                return fr_to_add
            }
        }
    } else {
        for (v1, v2) in qry_seq[..qry_seq.len()-v].iter().rev().zip(ref_seq[..ref_seq.len()-h].iter().rev()) {
            if *v1 == *v2 {
                fr_to_add += 1;
            } else {
                return fr_to_add
            }
        }
    }
    fr_to_add
}

#[inline]
fn dropout_wf_next(
    wf: &WaveFront,
    score: WfScore,
    penalties: &Penalties,
    ref_seq: &[u8],
    qry_seq: &[u8],
    ref_len: usize,
    qry_len: usize,
    forward: bool,
) -> (Option<WfK>, WfsComps) {
    let k_range_vector: Vec<i32> = {
        let max_k = wf[score].max_k;
        (-max_k..=max_k).collect()
    };
    /*
    [1] Wf Next
    */
    let mut wfs_components: WfsComps = {
        let mut wfs_components: WfsComps = vec![[Component::default(); 3]; k_range_vector.len()];

        // (1) From score: s-o-e
        if let Some(pre_score) = score.checked_sub(penalties.o+penalties.e) {
            let max_k = wf[pre_score].max_k;
            let pre_wfs = &wf[pre_score];
            for (res_index, k) in k_range_vector.iter().enumerate() {
                let comp_array = &mut wfs_components[res_index];
                // 1. Update I from M & M from I
                let mut comp_index = max_k + k - 1;
                if let Some([pre_m, _, _]) = pre_wfs.comp.get(comp_index as usize) {
                    if pre_m.bt != EMPTY {
                        // Update I
                        comp_array[1] = Component {
                            fr: pre_m.fr + 1,
                            bt: FROM_M,
                        };
                        
                    }
                }
                // 2. Update D from M & M from D
                comp_index += 2;
                if let Some([pre_m, _, _]) = pre_wfs.comp.get(comp_index as usize) {
                    if pre_m.bt != EMPTY {
                        // Update D
                        comp_array[2] = Component {
                            fr: pre_m.fr,
                            bt: FROM_M,
                        };
                    }
                }
            }
        }
        // (2) From score: s-e
        if let Some(pre_score) = score.checked_sub(penalties.e) {
            let pre_wfs = &wf[pre_score];
            k_range_vector.iter().enumerate().for_each(|(res_index, k)| {
                let comp_array = &mut wfs_components[res_index];
                // 1. Update I from I
                let mut comp_index = pre_wfs.max_k + k - 1;
                if let Some([_, pre_i, _]) = pre_wfs.comp.get(comp_index as usize) {
                    if pre_i.bt != EMPTY {
                        // Update I
                        if comp_array[1].bt == EMPTY || comp_array[1].fr > pre_i.fr + 1 {
                            comp_array[1] = Component {
                                fr: pre_i.fr + 1,
                                bt: FROM_I,
                            };
                        };
                    }
                }
                // 2. Update D from D
                comp_index += 2;
                if let Some([_, _, pre_d]) = pre_wfs.comp.get(comp_index as usize) {
                    if pre_d.bt != EMPTY {
                        // Update D
                        if comp_array[2].bt == EMPTY || comp_array[2].fr > pre_d.fr {
                            comp_array[2] = Component {
                                fr: pre_d.fr,
                                bt: FROM_D,
                            };
                        };
                    }
                }
            });
        }
        // (3) From score: s-x
        if let Some(pre_score) = score.checked_sub(penalties.x) {
            let pre_wfs = &wf[pre_score];
            k_range_vector.iter().enumerate().for_each(|(res_index, k)| {
                let comp_array = &mut wfs_components[res_index];
                // 1. Update M from M
                let comp_index = pre_wfs.max_k + k;
                if let Some([pre_m, _, _]) = pre_wfs.comp.get(comp_index as usize) {
                    // Update M
                    comp_array[0] = Component {
                        fr: pre_m.fr + 1,
                        bt: FROM_M,
                    };
                }
                // 2. Update M from I
                if comp_array[1].bt != EMPTY {
                    if comp_array[0].bt == EMPTY || comp_array[1].fr >= comp_array[0].fr {
                        comp_array[0] = Component {
                            fr: comp_array[1].fr,
                            bt: FROM_I,
                        };
                    };
                }
                // 3. Update M from D
                if comp_array[2].bt != EMPTY {
                    if comp_array[0].bt == EMPTY || comp_array[2].fr >= comp_array[0].fr {
                        comp_array[0] = Component {
                            fr: comp_array[2].fr,
                            bt: FROM_D,
                        };
                    };
                }
            });
        }    
        wfs_components
    };
    /*
    [2] Extend & check exit condition
    */
    for ([mcomp, _, _], k) in wfs_components.iter_mut().zip(k_range_vector.into_iter()) {
        if mcomp.bt != EMPTY {
            // Extend & update
            let mut v = (mcomp.fr - k) as usize;
            let mut h = mcomp.fr as usize;
            let fr_offset = seqs_compare(qry_seq, ref_seq, v, h, forward);
            mcomp.fr += fr_offset;
            // Check exit condition
            v += fr_offset as usize;
            h += fr_offset as usize;
            if h >= ref_len || v >= qry_len {
                return (Some(k), wfs_components);
            }
        };
    };
    (None, wfs_components)
}

pub type BacktraceResult = (Cigar, SequenceLength); // cigar is reversed 
// (anchor index, size, checkpoint k, checkpoint fr)
pub type AnchorsToPassCheck = Vec<(usize, i32, i32, i32)>;
// (length , penalty in ref)
pub type CigarReference = (usize, usize);
// key: index of anchor, val: CigarReference
pub type BacktraceRefed = HashMap<usize, CigarReference>;

#[inline]
pub fn dropout_wf_backtrace(
    wf: &WaveFront, penalties: &Penalties, mut score: WfScore, mut k: WfK,
    check_points_values: &AnchorsToPassCheck,
) -> (BacktraceResult, BacktraceRefed) {
    // INIT
    let mut operation_length: usize = 0;
    let mut to_check_index: HashSet<usize> = HashSet::from_iter(0..check_points_values.len());
    // FIXME: check if this cap is enough.
    let mut cigar: Cigar = Vec::with_capacity(score);
    let mut checkpoint_backtrace: BacktraceRefed = HashMap::with_capacity(check_points_values.len());
    
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
                            if let Some((Operation::Subst, last_fr)) = cigar.last_mut() {
                                *last_fr += 1;
                            } else {
                                cigar.push((Operation::Subst, 1));
                            }
                        } else {
                            cigar.push((Operation::Match, match_count));
                            cigar.push((Operation::Subst, 1));
                        }
                        operation_length += (match_count + 1) as usize;
                        // (8) Check if anchor is passed
                        for checkpoint_index in to_check_index.clone() {
                            let &(anchor_index, size, checkpoint_k, checkpoint_fr) = &check_points_values[checkpoint_index];
                            if (checkpoint_k == k) && (checkpoint_fr <= fr) && (checkpoint_fr - size >= next_fr) {
                                checkpoint_backtrace.insert(
                                    anchor_index,
                                    (
                                        operation_length - (checkpoint_fr - next_fr) as usize,
                                        score + penalties.x
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
                            cigar.push((Operation::Match, match_count));
                        }
                        operation_length += match_count as usize;
                        // (8) Check if anchor is passed
                        for checkpoint_index in to_check_index.clone() {
                            let &(anchor_index, size, checkpoint_k, checkpoint_fr) = &check_points_values[checkpoint_index];
                            if (checkpoint_k == k) && (checkpoint_fr <= fr) && (checkpoint_fr - size >= next_fr) {
                                checkpoint_backtrace.insert(
                                    anchor_index,
                                    (
                                        operation_length - (checkpoint_fr - next_fr) as usize,
                                        score
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
                            cigar.push((Operation::Match, match_count));
                        }
                        operation_length += match_count as usize;
                        // (8) Check if anchor is passed
                        for checkpoint_index in to_check_index.clone() {
                            let &(anchor_index, size, checkpoint_k, checkpoint_fr) = &check_points_values[checkpoint_index];
                            if (checkpoint_k == k) && (checkpoint_fr <= fr) && (checkpoint_fr - size >= next_fr) {
                                checkpoint_backtrace.insert(
                                    anchor_index,
                                    (
                                        operation_length - (checkpoint_fr - next_fr) as usize,
                                        score
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
                            cigar.push((Operation::Match, fr as OperationLength));
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
                        if let Some((Operation::Ins, last_fr)) = cigar.last_mut() {
                            *last_fr += 1;
                        } else {
                            cigar.push((Operation::Ins, 1));
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
                        if let Some((Operation::Ins, last_fr)) = cigar.last_mut() {
                            *last_fr += 1;
                        } else {
                            cigar.push((Operation::Ins, 1));
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
                        if let Some((Operation::Del, last_fr)) = cigar.last_mut() {
                            *last_fr += 1;
                        } else {
                            cigar.push((Operation::Del, 1));
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
                        if let Some((Operation::Del, last_fr)) = cigar.last_mut() {
                            *last_fr += 1;
                        } else {
                            cigar.push((Operation::Del, 1));
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

#[cfg(test)]
mod tests {
    use crate::*;
    use super::*;

    #[test]
    fn get_wf() {
        let penalties = Penalties {
            x: 2,
            o: 3,
            e: 1,
        };
        let score: usize = 10;
        let wf = penalties.wf_init(score);

        println!("score: {}, len: {}", score, wf.len());
        println!("wf: {:#?}", wf);
    }
}
