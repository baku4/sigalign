use std::collections::{HashMap, HashSet};

type WF = Vec<Option<WFscore>>; // Wavefront
type WFscore = [Option<Component>; 3]; // Wavefront of score
type Component = HashMap<i32, i32>; // MID Component k: k, v: f.r.point

fn wf_align(query: Vec<u8>, text: Vec<u8>, penalties: Vec<u8>) {
    let n = query.len();
    let m = text.len();
    // set offset
    let ak = (m-n) as i32;
    let ao = m as i32;
    // init
    let mut score: usize = 0;
    let mut wf = {
        let mut m_component: Component = HashMap::new();
        m_component.insert(0, 0);
        let wf_score: WFscore = [Some(m_component), None, None];
        vec![Some(wf_score)]
    };

    loop {
        // extend & exit condition
        if let Some(wf_score) = wf[score].as_mut() {
            if let Some(m_component) = wf_score[0].as_mut() {
                // extend
                wf_extend(m_component, &query, &text);
                // exit condition
                if let Some(fr_point) = m_component.get(&ak) {
                    if *fr_point >= ao {
                        break;
                    }
                }
            }
        }
        // next wf
        score += 1;
        wf_next(&mut wf, &query, &text, score);
    }
}

fn wf_extend(m_component: &mut Component, query: &Vec<u8>, text: &Vec<u8>) {
    for (k, fr_point) in m_component.iter_mut() {
        let mut v = (*fr_point - *k) as usize;
        let mut h = *fr_point as usize;
        while query[v] == text[h] {
            *fr_point += 1;
            v += 1;
            h += 1;
        }
    }
}

fn wf_next(wf: &mut WF, query: &Vec<u8>, text: &Vec<u8>, score: usize) {

}