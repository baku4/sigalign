use crate::io::cigar::{Cigar, Operation};
use super::{Penalties, AlignmentResult};

use std::collections::{HashSet, HashMap};
use std::iter::FromIterator;

// For WaveFront
type WF_SCORE = usize;
type WF_K = i32;
type FR_POINT = i32;

// For Backtrace
type BACTRACE = u8;
const EMPTY: u8 = 0;
const FROM_M: u8 = 1;
const FROM_I: u8 = 2;
const FROM_D: u8 = 3;
const START_POINT: u8 = 4;

#[derive(Clone, Default, Debug)]
pub struct WFS {
    m_fr: FR_POINT,
    i_fr: FR_POINT,
    d_fr: FR_POINT,
    m_bt: BACTRACE,
    i_bt: BACTRACE,
    d_bt: BACTRACE,
}

pub type WaveFront = Vec<WFS>;

// TODO: vector from raw
fn empty_wf(size: usize) -> WaveFront {
    unsafe {
        vec![std::mem::zeroed(); size]
    }
}

impl Penalties {
    #[inline]
    fn get_max_k(&self, score: WF_SCORE) -> usize {
        if score < self.o {
            1
        } else {
            (score - self.o) / self.e
        }
    }
    #[inline]
    fn get_index(&self, score: WF_SCORE, k: WF_K) -> usize {
        if score < self.o {
            score
        } else {
            let s_m_po = score - self.o;
            let q = s_m_po / self.e;
            let r = s_m_po % self.e;
            self.o + (q*q + q - 1)*self.e + (2*q-1)*r + q + k as usize
        }
    }
    #[inline]
    fn get_index_max_k(&self, score: WF_SCORE) -> (usize, WF_K) {
        if score < self.o {
            (score, 1)
        } else {
            let s_m_po = score - self.o;
            let q = s_m_po / self.e;
            let r = s_m_po % self.e;
            let start_index = self.o + (q*q + q - 1)*self.e + (2*q-1)*r;
            (start_index, q as WF_K)
        }
    }
    #[inline]
    fn get_wf_size(&self, panalty_spare: WF_SCORE) -> usize {
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
    fn get_pre_indices(&self, score: WF_SCORE) -> [(usize, i32); 3] {
        [
            self.get_index_max_k(score-self.o-self.x), // (1) score: s-o-e
            self.get_index_max_k(score-self.x), // (2) score: s-x
            self.get_index_max_k(score-self.e), // (3) score: s-e
        ]
    }
}

type WfRes = (WaveFront, usize, i32); // (WaveFront, Score, Last k)

pub fn dropout_wf_align(
    qry_seq: &[u8], ref_seq: &[u8],
    penalty_spare: usize, penalties: &Penalties,
) -> Result<WfRes, WaveFront> {
    let n = qry_seq.len();
    let m = ref_seq.len();

    // init
    let wf_size = penalties.get_wf_size(penalty_spare);
    let mut wf: WaveFront = empty_wf(wf_size);
    wf[0].m_bt = START_POINT;
    // first index & max k
    let (mut start_index, mut max_k) = (0, 0);

    // main
    for score in 0..penalty_spare {
        let last_index = start_index + (2*max_k as usize) + 1;
        // extend & check exit condition
        'wfs: for (wfs, k) in wf[start_index..last_index].iter_mut().zip(-max_k..=max_k) {
            /*
            (1) extend
            */
            if let EMPTY = wfs.m_bt {
                continue 'wfs;
            }
            let v = wfs.m_fr - k;
            let h = wfs.m_fr;
            let fr_to_add = compare_seqs(qry_seq, ref_seq, v as usize, h as usize);
            /*
            (2) check exit condition
            */
            if ((h + fr_to_add) as usize >= m) || ((v + fr_to_add) as usize >= n) {
                // Return Ok
                return Ok((wf, score, k)) 
            } else {
                wfs.m_fr += fr_to_add;
            }
        }
        // change to next index & max k
        {
            let temp = penalties.get_index_max_k(score + 1);
            start_index = temp.0;
            max_k = temp.1;
        }
        dropout_wf_next(&mut wf, penalties, score, start_index, max_k);
    };
    // Return Err
    Err(wf)
}

#[inline]
fn compare_seqs(qry_seq: &[u8], ref_seq: &[u8], v: usize, h: usize) -> i32 {
    let mut fr_to_add: i32 = 0;
    qry_seq[v..].iter().zip(ref_seq[h..].iter()).for_each(|(v1, v2)| {
        if *v1 == *v2 { fr_to_add += 1; }
    });
    fr_to_add
}

#[inline]
fn dropout_wf_next(
    wf: &mut WaveFront, penalties: &Penalties, score: usize, start_index: usize, max_k: i32
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
        let mut msk_array: [FR_POINT; 3] = [-1, -1, -1];
        let mut msk_exist = false;

        { // Next I
            let fr_from_m: Option<FR_POINT> = {
                let index_gap = maxk_soe + k - 1;
                if index_gap >= 0 {
                    let wfs = &wf[sidx_soe+index_gap as usize];
                    match wfs.m_bt {
                        EMPTY => None,
                        _ => Some(wfs.m_fr)
                    }
                } else {
                    None
                }
            };
            let fr_from_i: Option<FR_POINT> = {
                let index_gap = maxk_se + k - 1;
                if index_gap >= 0 {
                    let wfs = &wf[sidx_se+index_gap as usize];
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
                                Some(fr_m + 1)
                            } else {
                                let wfs = &mut wf[index];
                                wfs.i_fr = fr_i + 1;
                                wfs.i_bt = FROM_I;
                                msk_array[1] = fr_i + 1;
                                msk_exist = true;
                                Some(fr_i + 1)
                            }
                        },
                        None => {
                            let wfs = &mut wf[index];
                            wfs.i_fr = fr_m + 1;
                            wfs.i_bt = FROM_M;
                            msk_array[1] = fr_m + 1;
                            msk_exist = true;
                            Some(fr_m + 1)
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
                            Some(fr_i + 1)
                        },
                        None => {
                            None
                        }
                    }
                }
            }
        };
        { // Next D
            let fr_from_m: Option<FR_POINT> = {
                let index_gap = maxk_soe + k + 1;
                if index_gap <= max_k {
                    let wfs = &wf[sidx_soe+index_gap as usize];
                    match wfs.m_bt {
                        EMPTY => None,
                        _ => Some(wfs.m_fr)
                    }
                } else {
                    None
                }
            };
            let fr_from_d: Option<FR_POINT> = {
                let index_gap = maxk_se + k + 1;
                if index_gap <= max_k {
                    let wfs = &wf[sidx_se+index_gap as usize];
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
                                Some(fr_m)
                            } else {
                                let wfs = &mut wf[index];
                                wfs.d_fr = fr_d;
                                wfs.d_bt = FROM_D;
                                Some(fr_d)
                            }
                        },
                        None => {
                            let wfs = &mut wf[index];
                            wfs.d_fr = fr_m;
                            wfs.d_bt = FROM_M;
                            msk_array[2] = fr_m;
                            msk_exist = true;
                            Some(fr_m)
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
                            Some(fr_d)
                        },
                        None => {
                            None
                        }
                    }
                }
            }
        };
        { // Next M
            let index_gap = maxk_sx + k;
            let wfs = &wf[sidx_sx+index_gap as usize];
            match wfs.m_bt {
                EMPTY => {},
                _ => {
                    msk_array[0] = wfs.m_fr + 1;
                    msk_exist = true;
                }
            };
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
}

pub type AnchorsToPassCheck = Vec<(usize, i32, i32, i32)>; // (anchor index, size, checkpoint k, checkpoint fr)
pub type CigarReference = (usize, usize); // (length , penalty in ref)
pub type PassedAnchors = HashMap<usize, CigarReference>; // key: index of anchor, val: CigarReference

#[inline]
pub fn dropout_wf_backtrace(
    wf: &WaveFront, penalties: &Penalties, mut score: usize, mut k: i32,
    check_points_values: &AnchorsToPassCheck,
) -> (AlignmentResult, PassedAnchors) {
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
                        let match_count = (fr-next_fr) as u32;
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
                        let match_count = (fr-next_fr) as u32;
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
                        let match_count = (fr-next_fr) as u32;
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
                            cigar.push((Operation::Match, fr as u32));
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
            _ => {
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
                    },
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