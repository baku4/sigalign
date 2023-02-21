use crate::core::regulators::{
	Penalty, 
};
use super::{WaveFront, WaveEndPoint, WaveFrontScore, Components, Component, BackTraceMarker, MatchCounter};

impl WaveFront {
    pub fn align_right_to_end_point(
        &mut self,
        ref_seq: &[u8],
        qry_seq: &[u8],
        penalties: &Penalty,
        spare_penalty: usize,
    ) {
        self.align_to_end_point(ref_seq, qry_seq, penalties, spare_penalty, &consecutive_match_forward)
    }
    pub fn align_left_to_end_point(
        &mut self,
        ref_seq: &[u8],
        qry_seq: &[u8],
        penalties: &Penalty,
        spare_penalty: usize,
    ) {
        self.align_to_end_point(ref_seq, qry_seq, penalties, spare_penalty, &consecutive_match_reverse)
    }
    #[inline]
    fn align_to_end_point(
        &mut self,
        ref_seq: &[u8],
        qry_seq: &[u8],
        penalties: &Penalty,
        spare_penalty: usize,
        match_counter: MatchCounter,
    ) {
        let ref_len = ref_seq.len();
        let qry_len = qry_seq.len();

        let first_match_count = match_counter(ref_seq, qry_seq, 0, 0);

        self.wave_front_scores[0].add_first_components(first_match_count);

        if first_match_count as usize >= ref_len || first_match_count as usize >= qry_len {
            let end_point = WaveEndPoint { score: 0, k: Some(0) };
            self.end_point = end_point;
        } else {
            let end_point = self.fill_wave_front_scores_until_end(
                ref_seq,
                qry_seq,
                spare_penalty,
                penalties,
                match_counter
            );
            self.end_point = end_point;
        }
    }
    fn fill_wave_front_scores_until_end(
        &mut self,
        ref_seq: &[u8],
        qry_seq: &[u8],
        mut spare_penalty: usize,
        penalties: &Penalty,
        match_counter: MatchCounter,
    ) -> WaveEndPoint {
        if self.wave_front_scores.len() <= spare_penalty {
            spare_penalty = self.wave_front_scores.len() - 1;
        }
        for score in 1..=spare_penalty {
            self.update_components_of_next_wave_front_score(score, penalties);
           
            let optional_last_k = self.wave_front_scores[score].extend_components_until_end(ref_seq, qry_seq, match_counter);

            if let Some(last_k) = optional_last_k {
                return WaveEndPoint { score: score, k: Some(last_k) };
            }
        }

        WaveEndPoint { score: spare_penalty, k: None }
    }
    fn update_components_of_next_wave_front_score(
        &mut self,
        score: usize,
        penalties: &Penalty,
    ) {
        let mismatch_penalty = &penalties.x;
        let gap_open_penalty = &penalties.o;
        let gap_extend_penalty = &penalties.e;

        let max_k = self.wave_front_scores[score].max_k;
        let new_components_by_k = &self.wave_front_scores[score].components_by_k;
        // TODO: Faster init
        unsafe {
            new_components_by_k.iter().for_each(
                |v| *(v as *const Components as *mut Components) = Components::default()
            );
        }

        // (1) From score: s-o-e
        if let Some(pre_score) = score.checked_sub((gap_open_penalty + gap_extend_penalty) as usize) {
            let pre_wave_front_score = &self.wave_front_scores[pre_score];
            new_components_by_k.iter().enumerate().for_each(|(index_of_k, component)| {
                let k = index_of_k as i32 - max_k;
                let new_components_of_k = component as *const Components as *mut Components;
                // 1. Update I from M & M from I
                if let Some(pre_components) = pre_wave_front_score.components_of_k_checked(k-1) {
                    let pre_m_component = &pre_components.m;
                    if pre_m_component.bt != BackTraceMarker::Empty {
                        // Update I
                        unsafe {
                            (*new_components_of_k).i = Component {
                                fr: pre_m_component.fr + 1,
                                deletion_count: pre_m_component.deletion_count,
                                bt: BackTraceMarker::FromM,
                            };
                        }
                    }
                }
                // 2. Update D from M & M from D
                if let Some(pre_components) = pre_wave_front_score.components_of_k_checked(k+1) {
                    let pre_m_component = &pre_components.m;
                    if pre_m_component.bt != BackTraceMarker::Empty {
                        // Update D
                        unsafe {
                            (*new_components_of_k).d = Component {
                                fr: pre_m_component.fr,
                                deletion_count: pre_m_component.deletion_count + 1,
                                bt: BackTraceMarker::FromM,
                            };
                        }
                    }
                }
            });
        }
        // (2) From score: s-e
        if let Some(pre_score) = score.checked_sub(*gap_extend_penalty as usize) {
            let pre_wave_front_score = &self.wave_front_scores[pre_score];
            new_components_by_k.iter().enumerate().for_each(|(index_of_k, component)| {
                let k = index_of_k as i32 - max_k;
                let new_components_of_k = component as *const Components as *mut Components;
                // 1. Update I from I
                if let Some(pre_components) = pre_wave_front_score.components_of_k_checked(k-1) {
                    let pre_i_component = &pre_components.i;

                    if pre_i_component.bt != BackTraceMarker::Empty {
                        // Update I
                        unsafe {
                            if (*new_components_of_k).i.bt == BackTraceMarker::Empty || (*new_components_of_k).i.fr < pre_i_component.fr + 1 {
                                (*new_components_of_k).i = Component {
                                    fr: pre_i_component.fr + 1,
                                    deletion_count: pre_i_component.deletion_count,
                                    bt: BackTraceMarker::FromI,
                                };
                            }
                        };
                    }
                }
                // 2. Update D from D
                if let Some(pre_components) = pre_wave_front_score.components_of_k_checked(k+1) {
                    let pre_d_component = &pre_components.d;
                    if pre_d_component.bt != BackTraceMarker::Empty {
                        // Update D
                        unsafe {
                            if (*new_components_of_k).d.bt == BackTraceMarker::Empty || (*new_components_of_k).d.fr < pre_d_component.fr {
                                (*new_components_of_k).d = Component {
                                    fr: pre_d_component.fr,
                                    deletion_count: pre_d_component.deletion_count + 1,
                                    bt: BackTraceMarker::FromD,
                                };
                            };
                        }
                    }
                }
            });
        }
        // (3) From score: s-x
        if let Some(pre_score) = score.checked_sub(*mismatch_penalty as usize) {
            let pre_wave_front_score = &self.wave_front_scores[pre_score];
            new_components_by_k.iter().enumerate().for_each(|(index_of_k, component)| {
                let k = index_of_k as i32 - max_k;
                let new_components_of_k = component as *const Components as *mut Components;
                // 1. Update M from M
                let pre_component_index = (pre_wave_front_score.max_k + k) as usize;

                if let Some(pre_components) = pre_wave_front_score.components_by_k.get(pre_component_index) {
                    let pre_m_component = &pre_components.m;
                    // Update M
                    unsafe {
                        (*new_components_of_k).m = Component {
                            fr: pre_m_component.fr + 1,
                            deletion_count: pre_m_component.deletion_count,
                            bt: BackTraceMarker::FromM,
                        };
                    }
                }
                unsafe {
                    // 2. Update M from I
                    if (*new_components_of_k).i.bt != BackTraceMarker::Empty {
                        if (*new_components_of_k).m.bt == BackTraceMarker::Empty || (*new_components_of_k).i.fr >= (*new_components_of_k).m.fr {
                            (*new_components_of_k).m = Component {
                                fr: (*new_components_of_k).i.fr,
                                deletion_count: (*new_components_of_k).i.deletion_count,
                                bt: BackTraceMarker::FromI,
                            };
                        };
                    }
                    // 3. Update M from D
                    if (*new_components_of_k).d.bt != BackTraceMarker::Empty {
                        if (*new_components_of_k).m.bt == BackTraceMarker::Empty || (*new_components_of_k).d.fr >= (*new_components_of_k).m.fr {
                            (*new_components_of_k).m = Component {
                                fr: (*new_components_of_k).d.fr,
                                deletion_count: (*new_components_of_k).d.deletion_count,
                                bt: BackTraceMarker::FromD,
                            };
                        };
                    }
                }
            });
        }
    }
}

impl WaveFrontScore {
    fn add_first_components(&mut self, first_match_count: i32) {
        self.components_by_k = vec![Components::new_start_point(first_match_count)];
    }
    fn extend_components_until_end(
        &mut self,
        ref_seq: &[u8],
        qry_seq: &[u8],
        match_counter: MatchCounter,
    ) -> Option<i32> {
        for (components, k) in self.components_by_k.iter_mut().zip(-self.max_k..=self.max_k) {
            let m_component = &mut components.m;

            if m_component.bt != BackTraceMarker::Empty {
                // Extend & update
                let mut v = (m_component.fr - k) as usize;
                let mut h = m_component.fr as usize;
                let match_count = match_counter(ref_seq, qry_seq, v, h);
                m_component.fr += match_count;
                // Check exit condition
                v += match_count as usize;
                h += match_count as usize;
                if h >= ref_seq.len() || v >= qry_seq.len() {
                    return Some(k);
                }
            };
        }
        None
    }
}

//TODO: Apply SIMD
//TODO: Make inlined
fn consecutive_match_forward(ref_seq: &[u8], qry_seq: &[u8], v: usize, h: usize) -> i32 {
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
fn consecutive_match_reverse(ref_seq: &[u8], qry_seq: &[u8], v: usize, h: usize) -> i32 {
    let mut fr_to_add: i32 = 0;
    for (v1, v2) in qry_seq[..qry_seq.len()-v].iter().rev().zip(ref_seq[..ref_seq.len()-h].iter().rev()) {
        if *v1 == *v2 {
            fr_to_add += 1;
        } else {
            return fr_to_add
        }
    }
    fr_to_add
}
