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
                if new_fr >= 0 {
                    Some((*k - k_gap, new_fr, bt.clone()))
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
        wf_next(&mut wf, &qry_seq, &ref_seq, score, penalties);
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
        wf_next(&mut wf, &qry_seq, &ref_seq, score, penalties);
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

pub type CheckPointsValues = Vec<(usize, i32, i32)>; // (anchor index, checkpoint k, checkpoint fr)
// key: index of anchor
// val: reverse index & penalty
pub type ConnectedBacktrace = HashMap<usize, (usize, usize)>;

pub fn wf_backtrace(
    wf: &WF, penalties: &Scores, start_k: i32,
    check_points_values: &CheckPointsValues,
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
                            let &(anchor_index, checkpoint_k, checkpoint_fr) = &check_points_values[checkpoint_index];
                            if (checkpoint_k == k) && (checkpoint_fr <= fr) && (checkpoint_fr > component.0) {
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
                            let &(anchor_index, checkpoint_k, checkpoint_fr) = &check_points_values[checkpoint_index];
                            if (checkpoint_k == k) && (checkpoint_fr <= fr) && (checkpoint_fr > component.0) {
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
                            let &(anchor_index, checkpoint_k, checkpoint_fr) = &check_points_values[checkpoint_index];
                            if (checkpoint_k == k) && (checkpoint_fr <= fr) && (checkpoint_fr > component.0) {
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

// inheritance check:
// which anchor is inheritable
// return - key: anchor index, val: (score, checkpoint_k, checkpoint_ext_fr, checkpoint_fr)
pub fn wf_check_inheritable(
    wf: &WF, scores: &Scores, check_points_values: CheckPointsValues,
) -> HashMap<usize, (usize, i32, i32, i32)> {
    // k checklist
    let mut checklist_by_k: Vec<(i32, HashSet<(i32, usize)>)> = {
        // key: checkpoint k , val: list of (checkpoint fr, anchor index)
        let mut to_check_k_map: HashMap<i32, HashSet<(i32, usize)>> = HashMap::new();
        for (anchor_index, checkpoint_k,  checkpoint_fr) in check_points_values {
            match to_check_k_map.get_mut(&checkpoint_k) {
                Some(val) => {
                    val.insert((checkpoint_fr, anchor_index));
                },
                None => {
                    to_check_k_map.insert(
                        checkpoint_k, vec![(checkpoint_fr, anchor_index)].into_iter().collect()
                    );
                },
            }
        };
        let mut checklist_by_k: Vec<(i32, HashSet<(i32, usize)>)> = to_check_k_map.into_iter().map(
            |(k, val)| {
                (k, val)
            }
        ).collect();
        // sort by k
        checklist_by_k.sort_by(|a, b| {
            a.0.cmp(&b.0)
        });
        checklist_by_k
    };
    // valid checkpoints
    // key: anchor index, val: (score, checkpoint_k, checkpoint_ext_fr, checkpoint_fr)
    let mut valid_checkpoints: HashMap<usize, (usize, i32, i32, i32)> = HashMap::new();
    wf.iter().enumerate().for_each(|(score, wfs)| {
        if let Some(wfs) = wfs {
            if let Some(mcomp) = &wfs[0] {
                let mut checklist_index: usize = 0;
                let mut mcomp_index: usize = 0;
                loop {
                    // get mcomp values
                    let (key, val, backtrace) = match mcomp.0.get(mcomp_index) {
                        Some(v) => v,
                        None => {
                            break;
                        },
                    };
                    // check the backtrace
                    match checklist_by_k.get_mut(checklist_index) {
                        Some((k, set)) => {
                            let kgap = *key - *k;
                            if kgap > 0 {
                                // comp_k > checkpoint_k
                                checklist_index += 1;
                            } else if kgap == 0 {
                                // k matched!
                                if let Backtrace::M(from_m) = backtrace {
                                    match from_m {
                                        FromM::M => {
                                            for (fr, anchor_index) in set.clone() {
                                                // if current fr point >= checkpoint fr
                                                if *val >= fr {
                                                    let pre_score = score - scores.0;
                                                    let pre_mcomp = wf[pre_score].as_ref().unwrap()[0].as_ref().unwrap();
                                                    // if pre mcomp fr point < checkpoint fr: valid
                                                    let pre_fr = pre_mcomp.get_frpoint(*k).unwrap();
                                                    if pre_fr < fr {
                                                        // insert checkpoint
                                                        valid_checkpoints.insert(
                                                            anchor_index,
                                                            (score, *k, *val, fr)
                                                        );
                                                        // remove value from current set
                                                        set.remove(&(fr, anchor_index));
                                                    }
                                                }
                                            }
                                        },
                                        FromM::I => {
                                            for (fr, anchor_index) in set.clone() {
                                                // if current fr point >= checkpoint fr
                                                if *val >= fr {
                                                    let pre_icomp = wf[score].as_ref().unwrap()[1].as_ref().unwrap();
                                                    // if pre mcomp fr point < checkpoint fr: valid
                                                    let pre_fr = pre_icomp.get_frpoint(*k).unwrap();
                                                    if pre_fr < fr {
                                                        // insert checkpoint
                                                        valid_checkpoints.insert(
                                                            anchor_index,
                                                            (score, *k, *val, fr)
                                                        );
                                                        // remove value from current set
                                                        set.remove(&(fr, anchor_index));
                                                    }
                                                }
                                            }
                                        },
                                        FromM::D => {
                                            for (fr, anchor_index) in set.clone() {
                                                // if current fr point >= checkpoint fr
                                                if *val >= fr {
                                                    let pre_dcomp = wf[score].as_ref().unwrap()[2].as_ref().unwrap();
                                                    // if pre mcomp fr point < checkpoint fr: valid
                                                    let pre_fr = pre_dcomp.get_frpoint(*k).unwrap();
                                                    if pre_fr < fr {
                                                        // insert checkpoint
                                                        valid_checkpoints.insert(
                                                            anchor_index,
                                                            (score, *k, *val, fr)
                                                        );
                                                        // remove value from current set
                                                        set.remove(&(fr, anchor_index));
                                                    }
                                                }
                                            }
                                        },
                                        _ => {},
                                    }
                                }
                                checklist_index += 1;
                                mcomp_index += 1;
                            } else {
                                // comp_k < checkpoint_k
                                mcomp_index += 1;
                            }
                        },
                        None => {
                            break;
                        },
                    }
                }
            }
        }
    });
    // check inheritable
    for (anchor_index, (checkpoint_score, checkpoint_k, checkpoint_ext_fr, _)) in valid_checkpoints.clone() {
        // first indel point
        let mut indel_score = checkpoint_score + scores.1 + scores.2;
        match wf.get(indel_score) {
            Some(wfs_option) => {
                let [mcomp, icomp, dcomp] = wfs_option.as_ref().unwrap();
                // Check I
                // 1. fr == (checkpoint_ext_fr + 1)
                // 2. I's backtrace:M
                // 3. M;s backtrace:I
                let i_passed = {
                    let (fr, i_bt) = icomp.as_ref().unwrap().backtrace(checkpoint_k+1);
                    match i_bt {
                        Backtrace::I(from_i) => {
                            match from_i {
                                FromI::M => {
                                    if fr == checkpoint_ext_fr+1 {
                                        let (_, m_bt) = mcomp.as_ref().unwrap().backtrace(checkpoint_k+1);
                                        match m_bt {
                                            Backtrace::M(from_m) => {
                                                match from_m {
                                                    FromM::I => {
                                                        true
                                                    },
                                                    _ => false
                                                }
                                            },
                                            _ => false
                                        }
                                    } else {
                                        false
                                    }
                                },
                                _ => false
                            }
                        },
                        _ => false
                    }
                };
                // Check D
                // 1. fr == checkpoint_ext_fr
                // 2. D's backtrace:M
                // 3. M;s backtrace:D
                let d_passed = {
                    let (fr, d_bt) = dcomp.as_ref().unwrap().backtrace(checkpoint_k-1);
                    match d_bt {
                        Backtrace::D(from_d) => {
                            match from_d {
                                FromD::M => {
                                    if fr == checkpoint_ext_fr {
                                        let (_, m_bt) = mcomp.as_ref().unwrap().backtrace(checkpoint_k-1);
                                        match m_bt {
                                            Backtrace::M(from_m) => {
                                                match from_m {
                                                    FromM::D => {
                                                        true
                                                    },
                                                    _ => false
                                                }
                                            },
                                            _ => false
                                        }
                                    } else {
                                        false
                                    }
                                },
                                _ => false
                            }
                        },
                        _ => false
                    }
                };
                if !(i_passed && d_passed) {
                    // if not passed: remove
                    valid_checkpoints.remove(&anchor_index);
                    break;
                }
            },
            None => {
                break;
            },
        };
        // next indel points
        let mut ext_count = 2;
        indel_score += scores.2;
        while let Some(wfs_option) = wf.get(indel_score) {
            let [mcomp, icomp, dcomp] = wfs_option.as_ref().unwrap();
            // Check I
            // 1. fr == (checkpoint_ext_fr + ext_count)
            // 2. I's backtrace:I
            // 3. M;s backtrace:I
            let i_passed = {
                let (fr, i_bt) = icomp.as_ref().unwrap().backtrace(checkpoint_k+ext_count);
                match i_bt {
                    Backtrace::I(from_i) => {
                        match from_i {
                            FromI::I => {
                                if fr == checkpoint_ext_fr+ext_count {
                                    let (_, m_bt) = mcomp.as_ref().unwrap().backtrace(checkpoint_k+ext_count);
                                    match m_bt {
                                        Backtrace::M(from_m) => {
                                            match from_m {
                                                FromM::I => {
                                                    true
                                                },
                                                _ => false
                                            }
                                        },
                                        _ => false
                                    }
                                } else {
                                    false
                                }
                            },
                            _ => false
                        }
                    },
                    _ => false
                }
            };
            // Check D
            // 1. fr == checkpoint_ext_fr
            // 2. D's backtrace:D
            // 3. M;s backtrace:D
            let d_passed = {
                let (fr, d_bt) = dcomp.as_ref().unwrap().backtrace(checkpoint_k-ext_count);
                match d_bt {
                    Backtrace::D(from_d) => {
                        match from_d {
                            FromD::D => {
                                if fr == checkpoint_ext_fr {
                                    let (_, m_bt) = mcomp.as_ref().unwrap().backtrace(checkpoint_k-ext_count);
                                    match m_bt {
                                        Backtrace::M(from_m) => {
                                            match from_m {
                                                FromM::D => {
                                                    true
                                                },
                                                _ => false
                                            }
                                        },
                                        _ => false
                                    }
                                } else {
                                    false
                                }
                            },
                            _ => false
                        }
                    },
                    _ => false
                }
            };
            if !(i_passed && d_passed) {
                // if not passed: remove
                valid_checkpoints.remove(&anchor_index);
                break;
            }
            // indel extension
            indel_score += scores.2;
            ext_count += 1;
        }
    }
    valid_checkpoints
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
