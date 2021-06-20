use std::collections::{HashMap, HashSet};

type WF = Vec<Option<WFscore>>; // Wavefront
type WFscore = [Option<Component>; 3]; // Wavefront of score
#[derive(Debug)]
struct Component(Vec<(i32, i32, Backtrace)>); // MID Component k: k, v: f.r.point

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
    // small < large
    fn get_two_frpoint(&self, k_small: i32, k_large: i32) -> (Option<i32>, Option<i32>) {
        let mut fr_1: Option<i32> = None;
        let mut fr_2: Option<i32> = None;
        for (key, val, _) in &self.0 {
            let gap_1 =  *key - k_small; // lo to hi
            if gap_1 == 0 {
                fr_1 = Some(*val);
            } else if gap_1 > 0 {
                let gap_2 =  *key - k_large;
                if gap_2 == 0 {
                    fr_2 = Some(*val);
                } else if gap_2 > 0 {
                    return (fr_1, fr_2)
                }
            }
        }
        return (fr_1, fr_2)
    }
    fn get_hi_lo(&self) -> (i32, i32) {
        (self.0[0].0, self.0.last().unwrap().0)
    }
    fn backtrace(&self, k: i32) -> (i32, &Backtrace) {
        for (key, fr, bt) in &self.0 {
            if *key == k {
                return (*fr, bt)
            }
        }
        panic!("backtrace err");
    }
}

fn wf_align(query: Vec<u8>, text: Vec<u8>, penalties: (usize, usize, usize)) -> WF {
    // penalties: [x, o, e]
    let n = query.len();
    let m = text.len();
    // set offset
    let ak = m as i32 - n as i32;
    let ao = m as i32;
    // init
    let mut score: usize = 0;
    let mut wf = {
        let m_component = Component::new_with_first(0, 0);
        let wf_score: WFscore = [Some(m_component), None, None];
        vec![Some(wf_score)]
    };
    // FIXME:
    // #[cfg(test)]
    // let mut i = 1;
    loop {
        // extend & exit condition
        if let Some(wf_score) = wf[score].as_mut() {
            if let Some(m_component) = wf_score[0].as_mut() {
                // extend
                wf_extend(m_component, &query, &text);
                // exit condition
                if let Some(fr_point) = m_component.get_frpoint(ak) {
                    if fr_point >= ao {
                        break;
                    }
                }
            }
        }
        // FIXME:
        // #[cfg(test)]
        // {
        //     println!("{:?}\n\n", wf);
        //     i += 1;
        //     if i == 10 {
        //         break;
        //     }
        // }
        // next wf
        score += 1;
        wf_next(&mut wf, &query, &text, score, penalties);
    }
    wf
}

fn wf_extend(m_component: &mut Component, query: &Vec<u8>, text: &Vec<u8>) {
    for (k, fr_point, _) in m_component.0.iter_mut() {
        let mut v = (*fr_point - *k) as usize;
        let mut h = *fr_point as usize;
        loop {
            match query.get(v) {
                Some(q) => {
                    match text.get(h) {
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

fn wf_next(wf: &mut WF, query: &Vec<u8>, text: &Vec<u8>, score: usize, penalties: (usize, usize, usize)) {
    let mut hi_vec = Vec::with_capacity(4);
    let mut lo_vec = Vec::with_capacity(4);
    // (1) check M s-x
    let m_sx_comp: Option<&Component> = {
        if score >= penalties.0 {
            match &wf[score-penalties.0] {
                Some(wfs) => {
                    match &wfs[0] {
                        Some(comp) => {
                            let (hi, lo) = comp.get_hi_lo();
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
                            let (hi, lo) = comp.get_hi_lo();
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
                            let (hi, lo) = comp.get_hi_lo();
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
                            let (hi, lo) = comp.get_hi_lo();
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

fn wf_backtrace(
    wf: &mut WF, query: &Vec<u8>, text: &Vec<u8>,
    penalties: (usize, usize, usize),
    score: usize, ak: i32
){
    let start_component = wf[score].as_ref().unwrap()[0].as_ref().unwrap().backtrace(ak);
    loop {

    }
}

mod test {
    use super::*;
    #[test]
    fn test() {
        // query: Vec<u8>, text: Vec<u8>, penalties: (usize, usize, usize)
        let query = "GATACA".as_bytes().to_vec();
        let text = "GAGATA".as_bytes().to_vec();
        let penalties: (usize, usize, usize) =  (4, 6, 2);

        let wf = wf_align(query, text, penalties);

        println!("{:?}", wf);
    }
}