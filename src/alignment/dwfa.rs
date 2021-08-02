use crate::io::cigar::Cigar;
use super::Penalties;

use std::collections::HashMap;

type WF_SCORE = usize;
type WF_K = i32;
type FR_POINT = i32;

type BACTRACE = u8;
const EMPTY: u8 = 0;
const FROM_M: u8 = 1;
const FROM_I: u8 = 2;
const FROM_D: u8 = 3;
const START_POINT: u8 = 4;

#[derive(Clone, Default)]
pub struct WFS {
    m_fr: FR_POINT,
    i_fr: FR_POINT,
    d_fr: FR_POINT,
    m_bt: BACTRACE,
    i_bt: BACTRACE,
    d_bt: BACTRACE,
}

type WaveFront = Vec<WFS>;

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

pub fn dropout_wf_align(
    qry_seq: &[u8], ref_seq: &[u8],
    penalty_spare: usize, spl: f64,
    penalties: &Penalties,
) -> Result<(WaveFront, i32), WaveFront> {
    let n = qry_seq.len();
    let m = ref_seq.len();

    // init
    let wf_size = penalties.get_wf_size(penalty_spare);
    let mut wf: WaveFront = empty_wf(wf_size);
    wf[0].m_bt = START_POINT;
    // first index & max k
    let (mut start_index, mut max_k) = (0, 0);

    // main
    'score: for score in 0..penalty_spare {
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
                return Ok((wf, k))
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

pub type ChkpBacktrace = Vec<(usize, i32, i32, i32)>; // (anchor index, size, checkpoint k, checkpoint fr)
// key: index of anchor
// val: reverse index & penalty
pub type ConnectedBacktrace = HashMap<usize, (usize, usize)>;

#[inline]
fn dropout_wf_backtrace(
    wf: &WaveFront, penalties: &Penalties, start_k: i32,
    check_points_values: &ChkpBacktrace,
) -> () { // (Cigar, ConnectedBacktrace) {
    /*
    let mut operations: Vec<Operation> = Vec::new();
    let get_comp = |mat_idx: usize, s: usize, k: i32| wf[s].as_ref().unwrap()[mat_idx].as_ref().unwrap().backtrace(k);

    // init
    let mut s = wf.len() - 1;
    let mut k = start_k;
    let mut component = get_comp(0, s, k);
    // check points
    let mut to_check_index: HashSet<usize> = HashSet::from_iter(0..check_points_values.len());
    // let mut reverse_index: ReverseIndex = vec![None; check_points.len()];
    let mut reverse_index: ConnectedBacktrace = HashMap::new();

    loop {
        let fr = component.0;
        let bactrace = component.1;
        // Backtrace check
        match bactrace {
            Backtrace::M(from) => {
                // TODO: concat enums M,I,D
                match from {
                    FromM::M => {
                        // new comp
                        s -= penalties.0;
                        component = get_comp(0, s, k);
                        // extend operations
                        let mut new_ops = vec![Operation::Match; (fr-component.0-1) as usize];
                        new_ops.push(Operation::Subst);
                        operations.extend(new_ops);
                        // validation backtrace check point
                        for checkpoint_index in to_check_index.clone() {
                            let &(anchor_index, size, checkpoint_k, checkpoint_fr) = &check_points_values[checkpoint_index];
                            if (checkpoint_k == k) && (checkpoint_fr <= fr) && (checkpoint_fr - size >= component.0) {
                                reverse_index.insert(
                                    anchor_index,
                                    (
                                        operations.len() - (checkpoint_fr - component.0) as usize,
                                        s + penalties.0
                                    ),
                                );
                                to_check_index.remove(&checkpoint_index);
                            }
                        }
                    },
                    FromM::I => {
                        // new comp
                        component = get_comp(1, s, k);
                        // extend operations
                        operations.extend(vec![Operation::Match; (fr-component.0) as usize]);
                        // validation backtrace check point
                        for checkpoint_index in to_check_index.clone() {
                            let &(anchor_index, size, checkpoint_k, checkpoint_fr) = &check_points_values[checkpoint_index];
                            if (checkpoint_k == k) && (checkpoint_fr <= fr) && (checkpoint_fr - size >= component.0) {
                                reverse_index.insert(
                                    anchor_index,
                                    (
                                        operations.len() - (checkpoint_fr - component.0) as usize,
                                        s
                                    ),
                                );
                                to_check_index.remove(&checkpoint_index);
                            }
                        }
                    },
                    FromM::D => {
                        // new comp
                        component = get_comp(2, s, k);
                        // extend operations
                        operations.extend(vec![Operation::Match; (fr-component.0) as usize]);
                        // validation backtrace check point
                        for checkpoint_index in to_check_index.clone() {
                            let &(anchor_index, size, checkpoint_k, checkpoint_fr) = &check_points_values[checkpoint_index];
                            if (checkpoint_k == k) && (checkpoint_fr <= fr) && (checkpoint_fr - size >= component.0) {
                                reverse_index.insert(
                                    anchor_index,
                                    (
                                        operations.len() - (checkpoint_fr - component.0) as usize,
                                        s
                                    ),
                                );
                                to_check_index.remove(&checkpoint_index);
                            }
                        }
                    },
                    FromM::N => {
                        operations.extend(vec![Operation::Match; fr as usize]);
                        operations.reverse();
                        return (operations, reverse_index);
                    },
                }
            },
            Backtrace::I(from) => {
                match from {
                    FromI::M => {
                        s -= penalties.1 + penalties.2;
                        k -= 1;
                        // new comp
                        component = get_comp(0, s, k);
                        // extend operations
                        operations.push(Operation::Ins);
                    },
                    FromI::I => {
                        s -= penalties.2;
                        k -= 1;
                        // new comp
                        component = get_comp(1, s, k);
                        // extend operations
                        operations.push(Operation::Ins);
                    },
                }
            },
            Backtrace::D(from) => {
                match from {
                    FromD::M => {
                        s -= penalties.1 + penalties.2;
                        k += 1;
                        // new comp
                        component = get_comp(0, s, k);
                        // extend operations
                        operations.push(Operation::Del);
                    },
                    FromD::D => {
                        s -= penalties.2;
                        k += 1;
                        // new comp
                        component = get_comp(2, s, k);
                        // extend operations
                        operations.push(Operation::Del);
                    },
                }
            }
        }
    }
     */
}