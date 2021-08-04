use crate::{SequenceLength, OperationLength, Penalty};
use crate::io::cigar::{Cigar, Operation};
use super::{Penalties, Alignment};

use std::collections::{HashSet, HashMap};
use std::iter::FromIterator;

/* TYPES */
// WaveFront Align
type WfScore = usize;
type WfK = i32;
type FrPoint = i32;
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
    m_fr: FrPoint,
    i_fr: FrPoint,
    d_fr: FrPoint,
    m_bt: BackTrace,
    i_bt: BackTrace,
    d_bt: BackTrace,
}

#[repr(C)]
#[derive(Debug, Clone)]
struct Component {
    fr: FrPoint,
    bt: BackTrace,
}

impl Default for Component {
    fn default() -> Self {
        Self { fr: 0, bt: 0 }
    }
}

#[derive(Debug, Clone)]
pub struct WaveFrontScore {
    max_k: WfK,
    comp: Vec<[Component; 3]>,
}

impl WaveFrontScore {
    #[inline]
    fn with_max_k(max_k: usize) -> Self {
        Self {
            max_k: max_k as WfK,
            comp: vec![[Component::default(); 3]; 2*max_k+1],
        }
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

    // init
    let mut wf: WaveFront = penalties.wf_init(penalty_spare);
    wf[0].comp[0][0].bt = START_POINT;
    let fr_offset = compare_seqs(qry_seq, ref_seq, 0, 0);
    wf[0].comp[0][0].fr += fr_offset;
    if fr_offset as usize >= ref_len || fr_offset as usize >= qry_len {
        return Ok((wf,0,0))
    }

    // main
    for score in 0..=penalty_spare {
        if let Some(last_k) = dropout_wf_next(
            &mut wf[score],
            &wf,
            score,
            penalties,
            ref_seq,
            qry_seq,
            ref_len,
            qry_len
        ) {
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
    wfs: &mut WaveFrontScore,
    wf: & WaveFront,
    score: WfScore,
    penalties: &Penalties,
    ref_seq: &[u8],
    qry_seq: &[u8],
    ref_len: usize,
    qry_len: usize,
) -> Option<WfK> {
    for (curr_comp_index, components) in wfs.comp.iter_mut().enumerate() {
        // k
        let k = -wfs.max_k + curr_comp_index as i32;
        // Create components
        let mut m_comp: Option<Component> = None;
        let mut i_comp: Option<Component> = None;
        let mut d_comp: Option<Component> = None;
        
        /*
        [1] Wf Next
        */
        // (1) From score: s-o-e
        if let Some(pre_score) = score.checked_sub(penalties.o+penalties.e) {
            let pre_wfs = &wf[pre_score];
            // 1. Update I from M & M from I
            let mut comp_index = pre_wfs.max_k + k - 1;
            if let Some([pre_m, _, _]) = pre_wfs.comp.get(comp_index as usize) {
                if pre_m.bt != EMPTY {
                    // Update I
                    i_comp = Some(
                        Component {
                            fr: pre_m.fr + 1,
                            bt: FROM_M,
                        }
                    );
                    // Update M
                    m_comp = Some(
                        Component {
                            fr: pre_m.fr + 1,
                            bt: FROM_I,
                        }
                    );
                }
            }
            // 2. Update D from M & M from D
            comp_index += 2;
            if let Some([pre_m, _, _]) = pre_wfs.comp.get(comp_index as usize) {
                if pre_m.bt != EMPTY {
                    // Update D
                    d_comp = Some(
                        Component {
                            fr: pre_m.fr,
                            bt: FROM_M,
                        }
                    );
                    // Update M
                    if !match m_comp {
                        Some(ex_comp) => {
                            ex_comp.fr > pre_m.fr
                        },
                        None => false,
                    } {
                        m_comp = Some(
                            Component {
                                fr: pre_m.fr,
                                bt: FROM_D,
                            }
                        );
                    }
                }
            }
        }
        // (2) From score: s-e
        if let Some(pre_score) = score.checked_sub(penalties.e) {
            let pre_wfs = &wf[pre_score];
            // 1. Update I from I
            let mut comp_index = pre_wfs.max_k + k - 1;
            if let Some([_, pre_i, _]) = pre_wfs.comp.get(comp_index as usize) {
                if pre_i.bt != EMPTY {
                    // Update I
                    if !match i_comp {
                        Some(ex_comp) => {
                            ex_comp.fr > pre_i.fr + 1
                        },
                        None => false,
                    } {
                        i_comp = Some(
                            Component {
                                fr: pre_i.fr + 1,
                                bt: FROM_I,
                            }
                        )
                    }
                }
            }
            // 2. Update D from D
            comp_index += 2;
            if let Some([_, _, pre_d]) = pre_wfs.comp.get(comp_index as usize) {
                if pre_d.bt != EMPTY {
                    // Update D
                    if !match d_comp {
                        Some(ex_comp) => {
                            ex_comp.fr > pre_d.fr
                        },
                        None => false,
                    } {
                        d_comp = Some(
                            Component {
                                fr: pre_d.fr,
                                bt: FROM_D,
                            }
                        )
                    }
                }
            }
        }
        // (3) From score: s-x
        if let Some(pre_score) = score.checked_sub(penalties.x) {
            let pre_wfs = &wf[pre_score];
            // 1. Update M from M
            let comp_index = pre_wfs.max_k + k;
            if let Some([pre_m, _, _]) = pre_wfs.comp.get(comp_index as usize) {
                if pre_m.bt != EMPTY {
                    // Update M
                    if !match m_comp {
                        Some(ex_comp) => {
                            ex_comp.fr > pre_m.fr + 1
                        },
                        None => false,
                    } {
                        m_comp = Some(
                            Component {
                                fr: pre_m.fr + 1,
                                bt: FROM_M,
                            }
                        )
                    }
                }
            }
        }

        /*
        [2] Extend & check exit condition
        */
        // Update components
        if let Some(comp) = i_comp {
            components[1] = comp;
        }
        if let Some(comp) = d_comp {
            components[2] = comp;
        }
        if let Some(comp) = m_comp {
            // Extend & update
            let mut v = (comp.fr - k) as usize;
            let mut h = comp.fr as usize;
            let fr_offset = compare_seqs(qry_seq, ref_seq, v, h);
            comp.fr += fr_offset;
            components[0] = comp;
            // Check exit condition
            v += fr_offset as usize;
            h += fr_offset as usize;
            if h >= ref_len || v >= qry_len {
                return Some(k)
            }
        }
    }
    None
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
        let mut msk_array: [FrPoint; 3] = [NullFrPoint; 3];
        let mut msk_exist = false;

        // Next I
        let fr_from_m: Option<FrPoint> = {
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
        let fr_from_i: Option<FrPoint> = {
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
        let fr_from_m: Option<FrPoint> = {
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
        let fr_from_d: Option<FrPoint> = {
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

pub type BacktraceResult = (Cigar, SequenceLength, Penalty); // cigar is reversed 
// (anchor index, size, checkpoint k, checkpoint fr)
pub type AnchorsToPassCheck = Vec<(usize, i32, i32, i32)>;
// (length , penalty in ref)
pub type CigarReference = (usize, usize);
// key: index of anchor, val: CigarReference
pub type PassedAnchors = HashMap<usize, CigarReference>;

#[inline]
pub fn dropout_wf_backtrace_dep(
    wf: &WaveFrontDep, penalties: &Penalties, mut score: WfScore, mut k: WfK,
    check_points_values: &AnchorsToPassCheck,
) -> (BacktraceResult, PassedAnchors) {
    let mut opertion_length: usize = 0;
    let (start_index, max_k) = penalties.get_index_max_k(score);
    let mut index = start_index + (max_k + k) as usize;
    let mut component_type: usize = 0;
    let mut fr = wf[index].m_fr;

    let mut to_check_index: HashSet<usize> = HashSet::from_iter(0..check_points_values.len());
    // FIXME: check if this cap is enough.
    let mut cigar: Cigar = Vec::with_capacity(score);
    let mut checkpoint_backtrace: PassedAnchors = HashMap::with_capacity(check_points_values.len());

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
                        return ((cigar, opertion_length, score), checkpoint_backtrace,);
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
