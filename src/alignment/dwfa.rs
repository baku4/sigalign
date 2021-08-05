use crate::{SequenceLength, OperationLength, Penalty};
use crate::io::cigar::{Cigar, Operation};
use super::{Penalties, Alignment};

use std::collections::btree_map::Range;
use std::collections::{HashSet, HashMap};
use std::iter::FromIterator;
use std::slice::Iter;

/* TYPES */
// WaveFront Align
type WfScore = usize;
type WfK = i32;
type WfFrPoint = i32;
const NullMaxK: i32 = -1;
const NullFrPoint: i32 = -1;
// WaveFront Backtrace
type BackTrace = u8;
const EMPTY: u8 = 0;
const FROM_M: u8 = 1;
const FROM_I: u8 = 2;
const FROM_D: u8 = 3;
const START_POINT: u8 = 4;

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

pub type WaveFrontDep = Vec<WFS>;

pub type WaveFront = Vec<WaveFrontScore>;

// TODO: vector from raw
fn empty_wf_dep(size: usize) -> WaveFrontDep {
    unsafe {
        vec![std::mem::zeroed(); size]
    }
}

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
    #[inline]
    fn get_max_k(&self, score: WfScore) -> WfK {
        if score < self.o {
            0
        } else {
            ((score - self.o) / self.e) as WfK
        }
    }
    #[inline]
    fn get_index(&self, score: WfScore, k: WfK) -> usize {
        if score < self.o {
            score
        } else {
            let s_m_po = score - self.o;
            let q = s_m_po / self.e;
            let r = s_m_po % self.e;
            self.o + q*q*self.e + (2*q-1)*r + q + k as usize
        }
    }
    #[inline]
    fn get_index_max_k(&self, score: WfScore) -> (usize, WfK) {
        if score < self.o + self.e {
            (score, 0)
        } else {
            let s_m_po = score - self.o;
            let q = s_m_po / self.e;
            let r = s_m_po % self.e;
            let start_index = self.o + q*q*self.e + (2*q-1)*r;
            (start_index, q as WfK)
        }
    }
    #[inline]
    fn get_wf_size(&self, panalty_spare: WfScore) -> usize {
        if panalty_spare < self.o {
            panalty_spare + 1
        } else {
            let s_m_po = panalty_spare - self.o;
            let q = s_m_po / self.e;
            let r = s_m_po % self.e;
            self.o + (q*q + q - 1)*self.e + (2*q-1)*r + 2*q + 1
        }
    }
    #[inline]
    fn get_pre_indices(&self, score: WfScore) -> [(usize, WfK); 3] {
        [
            // (1) score: s-o-e
            if score < self.o + self.e {
                (0, NullMaxK)
            } else {
                self.get_index_max_k(score-self.o-self.e)
            },
            // (2) score: s-x
            if score < self.x {
                (0, NullMaxK)
            } else {
                self.get_index_max_k(score-self.x)
            },
            // (3) score: s-e
            if score < self.e {
                (0, NullMaxK)
            } else {
                self.get_index_max_k(score-self.e)
            },
        ]
    }
}

type WfResDrp = (WaveFrontDep, WfScore, WfK); // (WaveFront, Score, Last k)
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
    let fr_offset = compare_seqs(qry_seq, ref_seq, 0, 0);
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
            qry_len
        );
        // Update Component
        wf[score].comp = wfs_comps;
        if let Some(last_k) = exist_with_k {
            return Ok((wf, score, last_k));
        }
    };
    // Return Err
    Err(wf)
}

#[inline]
fn compare_seqs(qry_seq: &[u8], ref_seq: &[u8], v: usize, h: usize) -> i32 {
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

#[inline]
fn compare_seqs_dep(qry_seq: &[u8], ref_seq: &[u8], v: usize, h: usize) -> i32 {
    let mut fr_to_add: i32 = 0;
    qry_seq[v..].iter().zip(ref_seq[h..].iter()).for_each(|(v1, v2)| {
        if *v1 == *v2 { fr_to_add += 1; }
    });
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
                            
                        // Update M
                        comp_array[0] = Component {
                            fr: pre_m.fr + 1,
                            bt: FROM_I,
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
                        // Update M
                        if comp_array[0].bt == EMPTY || comp_array[0].fr > pre_m.fr {
                            comp_array[0] = Component {
                                fr: pre_m.fr,
                                bt: FROM_D,
                            };
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
                    if pre_m.bt != EMPTY {
                        // Update M
                        if comp_array[0].bt == EMPTY || comp_array[0].fr > pre_m.fr + 1 {
                            comp_array[0] = Component {
                                fr: pre_m.fr + 1,
                                bt: FROM_M,
                            };
                        };
                    }
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
            let fr_offset = compare_seqs(qry_seq, ref_seq, v, h);
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

#[inline]
fn dropout_wf_next_dep(
    wf: &mut WaveFrontDep, penalties: &Penalties, score: WfScore, start_index: usize, max_k: WfK
) {
    // Prepare indices of pre positions
    let [
        (sidx_soe, maxk_soe),
        (sidx_sx, maxk_sx),
        (sidx_se, maxk_se),
    ] = penalties.get_pre_indices(score);
    // wf next
    for k in -max_k..=max_k {
        let index = start_index + (max_k + k) as usize;
        let mut msk_array: [WfFrPoint; 3] = [NullFrPoint; 3];
        let mut msk_exist = false;

        // Next I
        let fr_from_m: Option<WfFrPoint> = {
            if maxk_soe >= 0 && maxk_soe >= (k-1).abs() {
                let wfs = &wf[sidx_soe + (maxk_soe + k - 1) as usize];
                match wfs.m_bt {
                    EMPTY => None,
                    _ => Some(wfs.m_fr)
                }
            } else {
                None
            }
        };
        let fr_from_i: Option<WfFrPoint> = {
            if maxk_se >= 0 && maxk_se >= (k-1).abs() {
                let wfs = &wf[sidx_se+(maxk_se + k - 1) as usize];
                match wfs.i_bt {
                    EMPTY => None,
                    _ => Some(wfs.i_fr)
                }
            } else {
                None
            }
        };
        match fr_from_m {
            Some(fr_m) => {
                match fr_from_i {
                    Some(fr_i) => {
                        if fr_m > fr_i {
                            let wfs = &mut wf[index];
                            wfs.i_fr = fr_m + 1;
                            wfs.i_bt = FROM_M;
                            msk_array[1] = fr_m + 1;
                            msk_exist = true;
                        } else {
                            let wfs = &mut wf[index];
                            wfs.i_fr = fr_i + 1;
                            wfs.i_bt = FROM_I;
                            msk_array[1] = fr_i + 1;
                            msk_exist = true;
                        }
                    },
                    None => {
                        let wfs = &mut wf[index];
                        wfs.i_fr = fr_m + 1;
                        wfs.i_bt = FROM_M;
                        msk_array[1] = fr_m + 1;
                        msk_exist = true;
                    }
                }
            },
            None => {
                match fr_from_i {
                    Some(fr_i) => {
                        let wfs = &mut wf[index];
                        wfs.i_fr = fr_i + 1;
                        wfs.i_bt = FROM_I;
                        msk_array[1] = fr_i + 1;
                        msk_exist = true;
                    },
                    None => ()
                }
            }
        }
        // Next D
        let fr_from_m: Option<WfFrPoint> = {
            if maxk_soe >= 0 && maxk_soe >= (k+1).abs(){
                let wfs = &wf[sidx_soe+(maxk_soe + k + 1) as usize];
                match wfs.m_bt {
                    EMPTY => None,
                    _ => Some(wfs.m_fr)
                }
            } else {
                None
            }
        };
        let fr_from_d: Option<WfFrPoint> = {
            if maxk_se >= 0 && maxk_se >= (k+1).abs(){
                let wfs = &wf[sidx_se+(maxk_se + k + 1) as usize];
                match wfs.d_bt {
                    EMPTY => None,
                    _ => Some(wfs.d_fr)
                }
            } else {
                None
            }
        };
        match fr_from_m {
            Some(fr_m) => {
                match fr_from_d {
                    Some(fr_d) => {
                        if fr_m > fr_d {
                            let wfs = &mut wf[index];
                            wfs.d_fr = fr_m;
                            wfs.d_bt = FROM_M;
                        } else {
                            let wfs = &mut wf[index];
                            wfs.d_fr = fr_d;
                            wfs.d_bt = FROM_D;
                        }
                    },
                    None => {
                        let wfs = &mut wf[index];
                        wfs.d_fr = fr_m;
                        wfs.d_bt = FROM_M;
                        msk_array[2] = fr_m;
                        msk_exist = true;
                    }
                }
            },
            None => {
                match fr_from_d {
                    Some(fr_d) => {
                        let wfs = &mut wf[index];
                        wfs.d_fr = fr_d;
                        wfs.d_bt = FROM_D;
                        msk_array[2] = fr_d;
                        msk_exist = true;
                    },
                    None => (),
                }
            }
        }
        // Next M
        if maxk_sx >= 0 && maxk_sx >= k.abs() {
            let wfs = &wf[sidx_sx+(maxk_sx + k) as usize];
            match wfs.m_bt {
                EMPTY => {},
                _ => {
                    msk_array[0] = wfs.m_fr + 1;
                    msk_exist = true;
                }
            };
        }
        if msk_exist {
            let (from, fr) = msk_array.iter().enumerate().max_by_key(|&(_, item)| item).unwrap();
            let current_wfs = &mut wf[index];
            match from {
                0 => {
                    current_wfs.m_fr = *fr;
                    current_wfs.m_bt = FROM_M;
                },
                1 => {
                    current_wfs.m_fr = *fr;
                    current_wfs.m_bt = FROM_I;
                },
                _ => {
                    current_wfs.m_fr = *fr;
                    current_wfs.m_bt = FROM_D;
                }
            }
        }
    }
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
                        score -= penalties.o + penalties.x;
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
                        score -= penalties.x;
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
                        score -= penalties.o + penalties.x;
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
                        score -= penalties.x;
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

#[inline]
pub fn dropout_wf_backtrace_dep(
    wf: &WaveFrontDep, penalties: &Penalties, mut score: WfScore, mut k: WfK,
    check_points_values: &AnchorsToPassCheck,
) -> (BacktraceResult, BacktraceRefed) {
    let mut opertion_length: usize = 0;
    let (start_index, max_k) = penalties.get_index_max_k(score);
    let mut index = start_index + (max_k + k) as usize;
    let mut component_type: usize = 0;
    let mut fr = wf[index].m_fr;

    let mut to_check_index: HashSet<usize> = HashSet::from_iter(0..check_points_values.len());
    // FIXME: check if this cap is enough.
    let mut cigar: Cigar = Vec::with_capacity(score);
    let mut checkpoint_backtrace: BacktraceRefed = HashMap::with_capacity(check_points_values.len());

    loop {
        let wfs = &wf[index];
        match component_type { // get bt
            0 => { // M
                match wfs.m_bt {
                    FROM_M => {
                        // (1) Next score
                        score -= penalties.x;
                        // (2) Next k
                        // not change
                        // (3) Next index
                        index = penalties.get_index(score, k);
                        // (4) Next fr
                        let next_fr = wf[index].m_fr;
                        // (5) Component type
                        // not chnage
                        // (6) Add Cigar
                        let match_count = (fr-next_fr) as OperationLength;
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
                        opertion_length += (match_count + 1) as usize;
                        // (7) Check anchor is passed
                        for checkpoint_index in to_check_index.clone() {
                            let &(anchor_index, size, checkpoint_k, checkpoint_fr) = &check_points_values[checkpoint_index];
                            if (checkpoint_k == k) && (checkpoint_fr <= fr) && (checkpoint_fr - size >= next_fr) {
                                checkpoint_backtrace.insert(
                                    anchor_index,
                                    (
                                        opertion_length - (checkpoint_fr - next_fr) as usize,
                                        score + penalties.x
                                    ),
                                );
                                to_check_index.remove(&checkpoint_index);
                            }
                        }
                        // (7) Next fr to fr
                        fr = next_fr;
                    },
                    FROM_I => {
                        // (1) Next score
                        // not change
                        // (2) Next k
                        // not change
                        // (3) Next index
                        // not change
                        // (4) Next fr
                        let next_fr = wf[index].i_fr;
                        // (5) Component type
                        component_type = 1;
                        // (6) Add Cigar
                        let match_count = (fr-next_fr) as OperationLength;
                        if match_count != 0 {
                            cigar.push((Operation::Match, match_count));
                        }
                        opertion_length += match_count as usize;
                        // (7) Check anchor is passed
                        for checkpoint_index in to_check_index.clone() {
                            let &(anchor_index, size, checkpoint_k, checkpoint_fr) = &check_points_values[checkpoint_index];
                            if (checkpoint_k == k) && (checkpoint_fr <= fr) && (checkpoint_fr - size >= next_fr) {
                                checkpoint_backtrace.insert(
                                    anchor_index,
                                    (
                                        opertion_length - (checkpoint_fr - next_fr) as usize,
                                        score
                                    ),
                                );
                                to_check_index.remove(&checkpoint_index);
                            }
                        }
                        // (7) Next fr to fr
                        fr = next_fr;
                    },
                    FROM_D => {
                        // (1) Next score
                        // not change
                        // (2) Next k
                        // not change
                        // (3) Next index
                        // not change
                        // (4) Next fr
                        let next_fr = wf[index].d_fr;
                        // (5) Component type
                        component_type = 2;
                        // (6) Add Cigar
                        let match_count = (fr-next_fr) as OperationLength;
                        if match_count != 0 {
                            cigar.push((Operation::Match, match_count));
                        }
                        opertion_length += match_count as usize;
                        // (7) Check anchor is passed
                        for checkpoint_index in to_check_index.clone() {
                            let &(anchor_index, size, checkpoint_k, checkpoint_fr) = &check_points_values[checkpoint_index];
                            if (checkpoint_k == k) && (checkpoint_fr <= fr) && (checkpoint_fr - size >= next_fr) {
                                checkpoint_backtrace.insert(
                                    anchor_index,
                                    (
                                        opertion_length - (checkpoint_fr - next_fr) as usize,
                                        score
                                    ),
                                );
                                to_check_index.remove(&checkpoint_index);
                            }
                        }
                        // (7) Next fr to fr
                        fr = next_fr;
                    },
                    _ => { // Can't be EMPTY -> this is START_POINT
                        if fr != 0 {
                            cigar.push((Operation::Match, fr as OperationLength));
                        };
                        // shrink
                        cigar.shrink_to_fit();
                        checkpoint_backtrace.shrink_to_fit();
                        return ((cigar, opertion_length), checkpoint_backtrace,);
                    }
                }
            },
            1 => { // I
                match wfs.i_bt {
                    FROM_M => {
                        // (1) Next score
                        score -= penalties.o + penalties.x;
                        // (2) Next k
                        k -= 1;
                        // not change
                        // (3) Next index
                        index = penalties.get_index(score, k);
                        // (4) Next fr
                        // cache not needed
                        // (5) Component type
                        component_type = 0;
                        // (6) Add Cigar
                        if let Some((Operation::Ins, last_fr)) = cigar.last_mut() {
                            *last_fr += 1;
                        } else {
                            cigar.push((Operation::Ins, 1));
                        }
                        opertion_length += 1;
                        // (7) Check anchor is passed
                        // not needed
                        // (7) Next fr to fr
                        fr = wf[index].i_fr;
                    },
                    _ => { // FROM_I
                        // (1) Next score
                        score -= penalties.x;
                        // (2) Next k
                        k -= 1;
                        // not change
                        // (3) Next index
                        index = penalties.get_index(score, k);
                        // (4) Next fr
                        // cache not needed
                        // (5) Component type
                        // not change
                        // (6) Add Cigar
                        if let Some((Operation::Ins, last_fr)) = cigar.last_mut() {
                            *last_fr += 1;
                        } else {
                            cigar.push((Operation::Ins, 1));
                        }
                        opertion_length += 1;
                        // (7) Check anchor is passed
                        // not needed
                        // (7) Next fr to fr
                        fr = wf[index].i_fr;
                    },
                }
            },
            _ => { // D
                match wfs.d_bt {
                    FROM_M => {
                        // (1) Next score
                        score -= penalties.o + penalties.x;
                        // (2) Next k
                        k += 1;
                        // not change
                        // (3) Next index
                        index = penalties.get_index(score, k);
                        // (4) Next fr
                        // cache not needed
                        // (5) Component type
                        component_type = 0;
                        // (6) Add Cigar
                        if let Some((Operation::Del, last_fr)) = cigar.last_mut() {
                            *last_fr += 1;
                        } else {
                            cigar.push((Operation::Del, 1));
                        }
                        opertion_length += 1;
                        // (7) Check anchor is passed
                        // not needed
                        // (7) Next fr to fr
                        fr = wf[index].d_fr;
                    }, // FROM_I
                    _ => {
                        // (1) Next score
                        score -= penalties.x;
                        // (2) Next k
                        k += 1;
                        // not change
                        // (3) Next index
                        index = penalties.get_index(score, k);
                        // (4) Next fr
                        // cache not needed
                        // (5) Component type
                        // not change
                        // (6) Add Cigar
                        if let Some((Operation::Del, last_fr)) = cigar.last_mut() {
                            *last_fr += 1;
                        } else {
                            cigar.push((Operation::Del, 1));
                        }
                        opertion_length += 1;
                        // (7) Check anchor is passed
                        // not needed
                        // (7) Next fr to fr
                        fr = wf[index].d_fr;
                    },
                }
            }
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
