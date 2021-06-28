use std::borrow::BorrowMut;
use std::collections::{HashMap, HashSet};
use std::iter::FromIterator;

use super::{Operation, Scores};

pub type WF = Vec<Option<WFscore>>; // Wavefront
type WFscore = [Option<Component>; 3]; // Wavefront of score

#[derive(Debug)]
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
}

pub type WFalignRes = (WF, i32);

pub fn dropout_wf_align(
    qry_seq: &[u8], ref_seq: &[u8], penalties: &Scores,
    panalty_spare: f64, spl: f64
) -> Result<WFalignRes, WF> {
    #[cfg(test)]
    {
        println!("panalty_spare: {}", panalty_spare);
    }
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
        wf_next(&mut wf, &qry_seq, &ref_seq, score, penalties);
    };
    // let operations = wf_backtrace(&mut wf, &qry_seq, &ref_seq, penalties, score, last_k);
    // Ok((operations, score, wf))
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

fn wf_next(wf: &mut WF, qry_seq: &[u8], ref_seq: &[u8], score: usize, penalties: &Scores) {
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
        if score >= (penalties.1 + penalties.2) {
            match &wf[score-(penalties.1 + penalties.2)] {
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

pub type CheckPoints = Vec<(i32, i32, i32)>; // (checkpoint k, checkpoint fr, size)
pub type ReverseIndex = Vec<Option<usize>>;

pub fn wf_backtrace(
    wf: &mut WF, penalties: &Scores, start_k: i32,
    check_points: &CheckPoints,
) -> (Vec<Operation>, ReverseIndex) {
    let mut operations: Vec<Operation> = Vec::new();
    let get_comp = |mat_idx: usize, s: usize, k: i32| wf[s].as_ref().unwrap()[mat_idx].as_ref().unwrap().backtrace(k);

    // init
    let mut s = wf.len() - 1;
    let mut k = start_k;
    let mut component = get_comp(0, s, k);
    // check points
    let mut to_check_index: HashSet<usize> = HashSet::from_iter(0..check_points.len());
    let mut reverse_index: ReverseIndex = vec![None; check_points.len()];

    loop {
        let fr = component.0;
        let bactrace = component.1;
        // Backtrace check
        match bactrace {
            Backtrace::M(from) => {
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
                            let &(checkpoint_k, checkpoint_fr, size) = &check_points[checkpoint_index];
                            if (checkpoint_k == k) && (checkpoint_fr + size <= fr) && (checkpoint_fr >= component.0) {
                                reverse_index[checkpoint_index] = Some(operations.len() - (component.0 - checkpoint_fr) as usize);
                                to_check_index.remove(&checkpoint_index);
                            }
                        }
                    },
                    FromM::I => {
                        // new comp
                        component = get_comp(1, s, k);
                        // extend operations
                        operations.extend(vec![Operation::Match; (fr-component.0) as usize]);
                    },
                    FromM::D => {
                        // new comp
                        component = get_comp(2, s, k);
                        // extend operations
                        operations.extend(vec![Operation::Match; (fr-component.0) as usize]);
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

pub fn wf_check_inheritable(wf: &WF, ref_pos_gap: i32, qry_pos_gap: i32, scores: &Scores) -> bool {
    let checkpoint_k = ref_pos_gap - qry_pos_gap;
    let (checkpoint_score, checkpoint_fr) = {
        let mut res: Option<(usize, i32)> = None;
        'wf_checker: for (score, wfs) in wf.iter().enumerate() {
            if let Some(wfs) = wfs {
                if let Some(mcomp) = &wfs[0] {
                    'fr_checker: for (key, val, backtrace) in &mcomp.0 {
                        let gap =  *key - checkpoint_k; // lo to hi
                        if gap > 0 {
                            break 'fr_checker;
                        } else if gap == 0 { // if matched k have fr_point
                            if *val >= ref_pos_gap {
                                if let Backtrace::M(from_m) = backtrace {
                                    match from_m {
                                        FromM::M => {
                                            res = Some((score, *val));
                                            break 'wf_checker;
                                        },
                                        _ => {
                                            break 'wf_checker;
                                        },
                                    }
                                }
                                break 'wf_checker;
                            } else {
                                // if fr point does not reach to the gap
                                break 'wf_checker;
                            }
                        }
                    }
                }
            }
        }
        match res {
            Some(v) => v,
            None => return false,
        }
    };
    let checkpoint_valid = {
        let pre_mcomp = wf[checkpoint_score - scores.0].as_ref().unwrap()[0].as_ref().unwrap();
        if pre_mcomp.get_frpoint(checkpoint_k).unwrap() < ref_pos_gap {
            // can reach
            true
        } else {
            // cannot reach
            false
        }
    };
    if checkpoint_valid {
        // checkpoint_score
        let first_indel_score = checkpoint_score + scores.1 + scores.2;
        // check first score
        match wf.get(first_indel_score) {
            Some(wfs_option) => {
                match wfs_option.as_ref() {
                    Some(wfs) => {
                        let icomp = &wfs[1];
                        let dcomp = &wfs[2];
                        //
                        false
                    },
                    None => {
                        false
                    }
                }
            },
            None => {
                return false
            },
        }
    } else {
        false
    }
}

fn wf_inheritance() {

}