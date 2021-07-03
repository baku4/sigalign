use std::borrow::BorrowMut;
use std::collections::{HashMap, HashSet};
use std::iter::FromIterator;

use super::{Operation, Scores};

pub type WF = Vec<Option<WFscore>>; // Wavefront
type WFscore = [Option<Component>; 3]; // Wavefront of score

#[derive(Debug, Clone)]
pub struct Component(Vec<(i32, i32, Backtrace)>); // MID Component k: k, v: f.r.point

#[derive(Debug, Clone)]
enum Backtrace {
    M(FromM),
    I(FromI),
    D(FromD),
}
#[derive(Debug, Clone)]
enum FromM {
    M,
    I,
    D,
    N,
}
#[derive(Debug, Clone)]
enum FromI {
    M,
    I,
}
#[derive(Debug, Clone)]
enum FromD {
    M,
    D,
}

impl Component {
    fn new() -> Self {
        Self(Vec::new())
    }
    fn new_with_first(k: i32, fr_point: i32) -> Self {
        Self(vec![(k, fr_point, Backtrace::M(FromM::N))])
    }
    fn get_frpoint(&self, k: i32) -> Option<i32> {
        for (key, val, _) in &self.0 {
            let gap =  *key - k; // lo to hi
            if gap > 0 {
                return None
            } else if gap == 0 {
                return Some(*val)
            }
        }
        return None
    }
    fn get_two_frpoint(&self, k_small: i32, k_large: i32) -> (Option<i32>, Option<i32>) {
        // k small < k large
        let mut fr_1: Option<i32> = None;
        let mut fr_2: Option<i32> = None;
        let mut skip_small = false;
        for (key, val, _) in &self.0 { // lo to hi
            if !skip_small {
                let gap =  *key - k_small;
                if gap > 0 {
                    skip_small = true;
                } else if gap == 0 {
                    fr_1 = Some(*val);
                    skip_small = true;
                }
            } 
            if skip_small {
                let gap =  *key - k_large;
                if gap > 0 {
                    return (fr_1, fr_2)
                } else if gap == 0 {
                    fr_2 = Some(*val);
                    return (fr_1, fr_2)
                }
            }
        }
        return (fr_1, fr_2)
    }
    fn get_k_hi_and_lo(&self) -> (i32, i32) {
        (self.0.last().unwrap().0, self.0[0].0)
    }
    fn backtrace(&self, k: i32) -> (i32, &Backtrace) {
        for (key, fr, bt) in &self.0 {
            if *key == k {
                return (*fr, bt)
            }
        }
        panic!("backtrace err");
    }
    fn check_exist_condition(&self, qry_len: i32, text_len: i32) -> Option<i32> {
        for (k, fr_point, _) in &self.0 {
            if (*fr_point >= text_len) || (*fr_point - *k >= qry_len as i32) {
                return Some(*k)
            }
        }
        return None
    }
    fn inherit(&self, k_gap: i32, fr_gap: i32) -> Option<Self> {
        let comp_vector: Vec<(i32, i32, Backtrace)> = self.0.iter().filter_map(
            |(k, fr, bt)| {
                let new_fr = *fr - fr_gap;
                let new_k = *k - k_gap;
                if (new_fr >= 0) && (new_fr >= new_k) {
                    Some((new_k, new_fr, bt.clone()))
                } else {
                    None
                }
            }
        ).collect();
        if comp_vector.len() == 0 {
            None
        } else {
            Some(
                Self(comp_vector)
            )
        }
    }
}

pub type WFalignRes = (WF, i32);

pub fn dropout_wf_align(
    qry_seq: &[u8], ref_seq: &[u8], penalties: &Scores,
    panalty_spare: f64, spl: f64
) -> Result<WFalignRes, WF> {
    // penalties: [x, o, e]
    let n = qry_seq.len();
    let m = ref_seq.len();
    // init
    let mut score: usize = 0;
    let mut wf = {
        let m_component = Component::new_with_first(0, 0);
        let wf_score: WFscore = [Some(m_component), None, None];
        vec![Some(wf_score)]
    };
    let last_k = loop {
        // extend & exit condition
        if let Some(wf_score) = wf[score].as_mut() {
            if let Some(m_component) = wf_score[0].as_mut() {
                // extend
                wf_extend(m_component, qry_seq, ref_seq);
                // exit condition
                if let Some(last_k) = m_component.check_exist_condition(n as i32, m as i32) {
                    break last_k;
                }
            }
        }
        score += 1;
        // check dropout
        if score as f64 - spl*((score as isize - penalties.1 as isize)/penalties.2 as isize) as f64 > panalty_spare {
            return Err(wf)
        }
        wf_next(&mut wf, score, penalties);
    };
    Ok((wf, last_k))
}

pub fn dropout_inherited_wf_align(
    inherited_wf: WF, qry_seq: &[u8], ref_seq: &[u8], penalties: &Scores,
    panalty_spare: f64, spl: f64
) -> Result<WFalignRes, WF> {
    let mut wf = inherited_wf;
    // penalties: [x, o, e]
    let n = qry_seq.len();
    let m = ref_seq.len();
    // init
    let mut score: usize = wf.len() - 1;
    let last_k = loop {
        score += 1;
        // check dropout
        if score as f64 - spl*((score as isize - penalties.1 as isize)/penalties.2 as isize) as f64 > panalty_spare {
            return Err(wf)
        }
        wf_next(&mut wf, score, penalties);
        // extend & exit condition
        if let Some(wf_score) = wf[score].as_mut() {
            if let Some(m_component) = wf_score[0].as_mut() {
                // extend
                wf_extend(m_component, qry_seq, ref_seq);
                // exit condition
                if let Some(last_k) = m_component.check_exist_condition(n as i32, m as i32) {
                    break last_k;
                }
            }
        }
    };
    Ok((wf, last_k))
}

fn wf_extend(m_component: &mut Component, qry_seq: &[u8], ref_seq: &[u8]) {
    for (k, fr_point, _) in m_component.0.iter_mut() {
        let mut v = (*fr_point - *k) as usize;
        let mut h = *fr_point as usize;
        loop {
            match qry_seq.get(v) {
                Some(q) => {
                    match ref_seq.get(h) {
                        Some(t) => {
                            if *q == *t {
                                *fr_point += 1;
                                v += 1;
                                h += 1;
                            } else {
                                break;
                            }
                        },
                        None => {
                            break;
                        }
                    }
                },
                None => {
                    break;
                }
            }
        }
    }
}

fn wf_next(wf: &mut WF, score: usize, penalties: &Scores) {
    let mut hi_vec = Vec::with_capacity(4);
    let mut lo_vec = Vec::with_capacity(4);
    // (1) check M s-x
    let m_sx_comp: Option<&Component> = {
        if score >= penalties.0 {
            match &wf[score-penalties.0] {
                Some(wfs) => {
                    match &wfs[0] {
                        Some(comp) => {
                            let (hi, lo) = comp.get_k_hi_and_lo();
                            hi_vec.push(hi);
                            lo_vec.push(lo);
                            Some(comp)
                        },
                        None => {
                            None
                        }
                    }
                },
                None => {
                    None
                },
            }
        } else {
            None
        }
    };
    // (2) check M s-o-e
    let m_soe_comp = {
        if score >= penalties.1 + penalties.2 {
            match &wf[score - penalties.1 - penalties.2] {
                Some(wfs) => {
                    match &wfs[0] {
                        Some(comp) => {
                            let (hi, lo) = comp.get_k_hi_and_lo();
                            hi_vec.push(hi);
                            lo_vec.push(lo);
                            Some(comp)
                        },
                        None => {
                            None
                        }
                    }
                },
                None => {
                    None
                },
            }
        } else {
            None
        }
    };
    // (3) check I s-e & D s-e
    let (i_se_comp, d_se_comp) = {
        if score >= penalties.2 {
            match &wf[score-penalties.2] {
                Some(wfs) => {
                    // I
                    let i_se_comp = match &wfs[1] {
                        Some(comp) => {
                            let (hi, lo) = comp.get_k_hi_and_lo();
                            hi_vec.push(hi);
                            lo_vec.push(lo);
                            Some(comp)
                        },
                        None => {
                            None
                        }
                    };
                    // D
                    let d_se_comp = match &wfs[2] {
                        Some(comp) => {
                            let (hi, lo) = comp.get_k_hi_and_lo();
                            hi_vec.push(hi);
                            lo_vec.push(lo);
                            Some(comp)
                        },
                        None => {
                            None
                        }
                    };
                    (i_se_comp, d_se_comp)
                },
                None => {
                    (None, None)
                },
            }
        } else {
            (None, None)
        }
    };
    // get hi & lo k
    let hi = match hi_vec.iter().max() {
        Some(v) => {
            *v + 1
        },
        None => {
            wf.push(None);
            return
        },
    };
    let lo = match lo_vec.iter().min() {
        Some(v) => {
            *v - 1
        },
        None => {
            return
        }
    };
    // next components
    let mut m_comp: Component = Component::new();
    let mut i_comp: Component = Component::new();
    let mut d_comp: Component = Component::new();
    for k in lo..=hi {
        let mut isk_vec: Vec<(i32, Backtrace)> = Vec::with_capacity(2);
        let mut dsk_vec: Vec<(i32, Backtrace)> = Vec::with_capacity(2);
        let mut msk_vec: Vec<(i32, Backtrace)> = Vec::with_capacity(3);   
        // M s-o-e comp
        if let Some(comp) = m_soe_comp {
            // fr_i: M s-o-e,k-1
            // fr_d: M s-o-e,k+1
            let (fr_i, fr_d) = comp.get_two_frpoint(k-1, k+1);
            if let Some(frpoint) = fr_i {
                isk_vec.push((frpoint+1, Backtrace::I(FromI::M)));
            }
            if let Some(frpoint) = fr_d {
                dsk_vec.push((frpoint, Backtrace::D(FromD::M)));
            }
        }
        // I s-e comp
        if let Some(comp) = i_se_comp {
            if let Some(frpoint) = comp.get_frpoint(k-1) {
                isk_vec.push((frpoint+1, Backtrace::I(FromI::I)));
            }
        }
        // D s-e comp
        if let Some(comp) = d_se_comp {
            if let Some(frpoint) = comp.get_frpoint(k+1) {
                dsk_vec.push((frpoint, Backtrace::D(FromD::D)));
            }
        }
        // M s-x comp
        if let Some(comp) = m_sx_comp {
            if let Some(frpoint) = comp.get_frpoint(k) {
                msk_vec.push((frpoint+1, Backtrace::M(FromM::M)));
            }
        }
        // isk
        if let Some((fr, backtrace)) = isk_vec.iter().max_by_key(|x| x.0) {
            msk_vec.push((*fr, Backtrace::M(FromM::I)));
            i_comp.0.push((k, *fr, backtrace.clone()));
        }
        // dsk
        if let Some((fr, backtrace)) = dsk_vec.iter().max_by_key(|x| x.0) {
            msk_vec.push((*fr, Backtrace::M(FromM::D)));
            d_comp.0.push((k, *fr, backtrace.clone()));
        }
        // msk
        if let Some((fr, backtrace)) = msk_vec.iter().max_by_key(|x| x.0) {
            m_comp.0.push((k, *fr, backtrace.clone()));
        }
    }
    // generate next wfs
    let mut next_wf_score: WFscore = [None, None, None];
    if m_comp.0.len() != 0 {
        next_wf_score[0] = Some(m_comp)
    }
    if i_comp.0.len() != 0 {
        next_wf_score[1] = Some(i_comp)
    }
    if d_comp.0.len() != 0 {
        next_wf_score[2] = Some(d_comp)
    }
    // save wfs to wf
    if next_wf_score.iter().all(|x| x.is_none()) {
        wf.push(None)
    } else {
        wf.push(Some(next_wf_score))
    }
}

pub type ChkpBacktrace = Vec<(usize, i32, i32, i32)>; // (anchor index, size, checkpoint k, checkpoint fr)
// key: index of anchor
// val: reverse index & penalty
pub type ConnectedBacktrace = HashMap<usize, (usize, usize)>;

pub fn wf_backtrace(
    wf: &WF, penalties: &Scores, start_k: i32,
    check_points_values: &ChkpBacktrace,
) -> (Vec<Operation>, ConnectedBacktrace) {
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
}

// anchor index, (size, checkpoint k, checkpoint fr, checkpoint extended fr)
pub type ChkpInherit = HashMap<usize, (usize, i32, i32, i32)>;
// inheritance check:
// which anchor is inheritable
// return - key: anchor index, val: (score, checkpoint k, checkpoint fr, checkpoint extended fr)
pub fn wf_check_inheritable(
    wf: &WF, scores: &Scores, check_points_values: ChkpInherit,
) -> ChkpInherit { // HashMap<usize, (usize, i32, i32, i32)> {
    let mut check_points_values = check_points_values;
    let get_comp = |mat_idx: usize, s: usize, k: i32| wf[s].as_ref().unwrap()[mat_idx].as_ref().unwrap().backtrace(k);
    wf.iter().enumerate().for_each(|(score, wfs_option)| {
        if let Some([Some(mcomp), icomp, dcomp]) = wfs_option {
            for (k, fr, bt) in &mcomp.0 {
                for (anchor_index, (size, checkpoint_k, checkpoint_fr, checkpoint_ext_fr)) in check_points_values.clone() {
                    let valid: bool = {
                        let k_gap_w_chkp = *k - checkpoint_k;
                        let fr_gap_w_chkp = *fr - checkpoint_ext_fr;
                        if k_gap_w_chkp == 0 && fr_gap_w_chkp == 0 {
                            // Conditiono 1: matched with checkpoint
                            if let Backtrace::M(m_bt) = bt {
                                let pre_fr = match m_bt {
                                    FromM::M => {
                                        let (fr, _) = get_comp(0, score - scores.0, *k);
                                        fr
                                    },
                                    FromM::I | FromM::D => {
                                        icomp.as_ref().unwrap().get_frpoint(*k).unwrap()
                                    },
                                    FromM::N => {
                                        0_i32
                                    }
                                };
                                if pre_fr <= checkpoint_fr - size as i32 {
                                    // change size to score
                                    check_points_values.get_mut(&anchor_index).unwrap().0 = score;
                                    true
                                } else {
                                    false
                                }
                            } else {
                                false
                            }
                        } else if fr_gap_w_chkp >= 0 && fr_gap_w_chkp >= k_gap_w_chkp {
                            // Condition 2: not inbound to outbound
                            let pre_comp = if let Backtrace::M(m_bt) = bt {
                                match m_bt {
                                    FromM::M => {
                                        wf[score - scores.0].as_ref().unwrap()[0].as_ref().unwrap()
                                    },
                                    FromM::I => icomp.as_ref().unwrap(),
                                    FromM::D => dcomp.as_ref().unwrap(),
                                    FromM::N => panic!(""),
                                }
                            } else {
                                // TODO: err msg
                                panic!("");
                            };
                            // let k_gap_w_pre = *k - *checkpoint_k;
                            let (pre_fr, pre_bt) = pre_comp.backtrace(*k);
                            let mut fr_gap_w_pre = *fr - pre_fr;
                            let k_gap_w_pre = match pre_bt {
                                Backtrace::M(_) => {
                                    0
                                },
                                Backtrace::I(_) => {
                                    fr_gap_w_pre -= 1;
                                    -1
                                },
                                Backtrace::D(_) => {
                                    1
                                },
                            };
                            // outbound check
                            if fr_gap_w_pre >= 0 && fr_gap_w_pre >= k_gap_w_pre {
                                true
                            } else {
                                false
                            }
                        } else {
                            true
                        }
                    };
                    if !valid {
                        check_points_values.remove(&anchor_index);
                    }
                }
            }
        }
    });
    check_points_values
}
pub fn wf_inherited_cache(wf: &WF, score: usize, k_gap: i32, fr_gap: i32) -> WF {
    let mut new_wf: WF = wf[score..].iter().map(
        |wfs_option| {
            match wfs_option.as_ref() {
                Some(wfs) => {
                    let mut new_wfs: [Option<Component>; 3] = [None, None, None];
                    for (idx, comp_option) in wfs.iter().enumerate() {
                        if let Some(comp) = comp_option {
                            new_wfs[idx] = comp.inherit(k_gap, fr_gap);
                        };
                    }
                    if new_wfs.iter().any(|x| {
                        match x {
                            Some(_) => true,
                            None => false,
                        }
                    }) {
                        Some(new_wfs)
                    } else {
                        None
                    }
                },
                None => {
                    None
                }
            }
        }
    ).collect();
    // change the first wfs
    let mut fisrt_wfs = new_wf[0].take().unwrap();
    {
        fisrt_wfs[1] = None;
        fisrt_wfs[2] = None;
        let fr = fisrt_wfs[0].take().unwrap().get_frpoint(0).unwrap();
        fisrt_wfs[0] = {
            let mut first_mcomp = Component::new();
            first_mcomp.0 = vec![(0_i32, fr, Backtrace::M(
                FromM::N
            ))];
            Some(first_mcomp)
        };
    }
    new_wf[0] = Some(fisrt_wfs);
    new_wf
}
