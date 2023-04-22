use crate::core::regulators::{
	Penalty, 
};
use super::{
    WaveFront, WaveEndPoint, WaveFrontScore, Components, Component, BackTraceMarker,
    MatchCounter, ForwardMatchCounter, ReverseMatchCounter,
};

impl WaveFront {
    #[inline]
    pub fn align_right_to_end_point(
        &mut self,
        tgt_seq: &[u8],
        qry_seq: &[u8],
        penalties: &Penalty,
        spare_penalty: u32,
    ) {
        self.align_to_end_point::<ForwardMatchCounter>(tgt_seq, qry_seq, penalties, spare_penalty)
    }
    #[inline]
    pub fn align_left_to_end_point(
        &mut self,
        tgt_seq: &[u8],
        qry_seq: &[u8],
        penalties: &Penalty,
        spare_penalty: u32,
    ) {
        self.align_to_end_point::<ReverseMatchCounter>(tgt_seq, qry_seq, penalties, spare_penalty)
    }
    #[inline]
    fn align_to_end_point<C: MatchCounter>(
        &mut self,
        tgt_seq: &[u8],
        qry_seq: &[u8],
        penalties: &Penalty,
        spare_penalty: u32,
    ) {
        let tgt_len = tgt_seq.len();
        let qry_len = qry_seq.len();

        let first_match_count = C::count_consecutive_match(tgt_seq, qry_seq, 0, 0);

        self.wave_front_scores[0].add_first_components(first_match_count);

        if first_match_count as usize >= tgt_len || first_match_count as usize >= qry_len {
            let end_point = WaveEndPoint { penalty: 0, k: Some(0) };
            self.end_point = end_point;
        } else {
            let end_point = self.fill_wave_front_scores_until_end::<C>(
                tgt_seq,
                qry_seq,
                spare_penalty,
                penalties,
            );
            self.end_point = end_point;
        }
    }
    #[inline]
    fn fill_wave_front_scores_until_end<C: MatchCounter>(
        &mut self,
        tgt_seq: &[u8],
        qry_seq: &[u8],
        mut spare_penalty: u32,
        penalties: &Penalty,
    ) -> WaveEndPoint {
        if self.wave_front_scores.len() as u32 <= spare_penalty {
            spare_penalty = (self.wave_front_scores.len() - 1) as u32;
        }
        for penalty in 1..=spare_penalty {
            self.update_components_of_next_wave_front_score(penalty, penalties);
           
            let optional_last_k = self.wave_front_scores[penalty as usize].extend_components_until_end::<C>(tgt_seq, qry_seq);

            if let Some(last_k) = optional_last_k {
                return WaveEndPoint { penalty: penalty as usize, k: Some(last_k) };
            }
        }

        WaveEndPoint { penalty: spare_penalty as usize, k: None }
    }
    #[inline]
    fn update_components_of_next_wave_front_score(
        &mut self,
        penalty: u32,
        penalties: &Penalty,
    ) {
        let mismatch_penalty = &penalties.x;
        let gap_open_penalty = &penalties.o;
        let gap_extend_penalty = &penalties.e;

        let max_k = self.wave_front_scores[penalty as usize].max_k;
        let new_components_by_k = &self.wave_front_scores[penalty as usize].components_by_k;
        unsafe {
            let ptr = new_components_by_k.as_ptr() as *mut u8;
            let byte_count = new_components_by_k.len() * std::mem::size_of::<Components>();
            std::ptr::write_bytes(ptr, 0, byte_count);
        }

        // (1) From score: s-o-e
        if let Some(pre_score) = penalty.checked_sub(gap_open_penalty + gap_extend_penalty) {
            let pre_wave_front_score = &self.wave_front_scores[pre_score as usize];
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
                                traversed: false,
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
                                traversed: false,
                            };
                        }
                    }
                }
            });
        }
        // (2) From score: s-e
        if let Some(pre_score) = penalty.checked_sub(*gap_extend_penalty) {
            let pre_wave_front_score = &self.wave_front_scores[pre_score as usize];
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
                                    traversed: false,
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
                                    traversed: false,
                                };
                            };
                        }
                    }
                }
            });
        }
        // (3) From score: s-x
        if let Some(pre_score) = penalty.checked_sub(*mismatch_penalty) {
            let pre_wave_front_score = &self.wave_front_scores[pre_score as usize];
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
                            traversed: false,
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
                                traversed: false,
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
                                traversed: false,
                            };
                        };
                    }
                }
            });
        }
    }
}

impl WaveFrontScore {
    #[inline]
    fn add_first_components(&mut self, first_match_count: i32) {
        self.components_by_k = vec![Components::new_start_point(first_match_count)];
    }
    #[inline]
    fn extend_components_until_end<C: MatchCounter>(
        &mut self,
        tgt_seq: &[u8],
        qry_seq: &[u8],
    ) -> Option<i32> {
        for (components, k) in self.components_by_k.iter_mut().zip(-self.max_k..=self.max_k) {
            let m_component = &mut components.m;

            if m_component.bt != BackTraceMarker::Empty {
                // Extend & update
                let mut v = (m_component.fr - k) as usize;
                let mut h = m_component.fr as usize;
                let match_count = C::count_consecutive_match(tgt_seq, qry_seq, v, h);
                m_component.fr += match_count;
                // Check exit condition
                v += match_count as usize;
                h += match_count as usize;
                if h >= tgt_seq.len() || v >= qry_seq.len() {
                    return Some(k);
                }
            };
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_if_the_default_value_of_component_is_filled_with_zero() {
        let components = Components::default();
        unsafe {
            let my_struct_bytes: [u8; std::mem::size_of::<Components>()] = std::mem::transmute(components);
            let all_zero = my_struct_bytes.iter().all(|&byte| byte == 0);
    
            assert!(all_zero, "Not all bytes in the struct are zero");
        }

        println!("All bytes in the struct are zero")
    }
}
