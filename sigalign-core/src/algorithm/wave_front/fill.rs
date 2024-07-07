use crate::core::regulators::Penalty;
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

        // (1) Initialize the first wave front score
        let first_match_count = C::count_consecutive_match(qry_seq, tgt_seq, 0, 0);
        self.wave_front_scores[0].add_first_components(first_match_count);

        // (2) Check if the end point is already reached
        if first_match_count as usize == tgt_len || first_match_count as usize == qry_len {
            let end_point = WaveEndPoint { penalty: 0, k: Some(0) };
            self.end_point = end_point;
        } else {
            // (3) Fill the wave front scores until the end point
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
        // This step is needed to occasionally over-estimate the spare penalty
        //   - WFS length is more accurate than spare penalty function
        if self.wave_front_scores.len() as u32 <= spare_penalty {
            spare_penalty = (self.wave_front_scores.len() - 1) as u32;
        }
        for penalty in 1..=spare_penalty {
            self.update_components_of_next_wave_front_score(penalty, penalties);

            let optional_last_k = self.wave_front_scores[penalty as usize].extend_m_components_to_the_end::<C>(tgt_seq, qry_seq);

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

        let (
            max_k,
            num_components,
            new_components_ptr,
        ) = {
            let next_wave_front_score = &mut self.wave_front_scores[penalty as usize];
            (
                next_wave_front_score.max_k,
                next_wave_front_score.components_by_k.len(),
                next_wave_front_score.components_by_k.as_mut_ptr(),
            )
        };

        // Initialize the components (components_by_k in WaveFrontScore) with all zero
        // If this step is not done, next components refer invalid values
        unsafe {
            let ptr = new_components_ptr as *mut u8;
            let byte_count = num_components * std::mem::size_of::<Components>();
            std::ptr::write_bytes(ptr, 0, byte_count);
        }

        // Update components of next wave front score
        //   1. Update I, D from previous M
        //   2. Update I from previous I, D from previous D
        //   3. Update M from previous M or current D, I

        // (1) From score: s-o-e
        // New insertion or deletion
        // TODO: Check if using i32 for penalty is more efficient than u32
        if let Some(pre_score) = penalty.checked_sub(gap_open_penalty + gap_extend_penalty) {
            let pre_wave_front_score = &self.wave_front_scores[pre_score as usize];
            for index_of_k in 0..num_components {
                let k = index_of_k as i32 - max_k;
                let new_components_of_k = unsafe { new_components_ptr.add(index_of_k) };
                // 1. Update D from previous M
                // TODO: Can be all components from previous wave front score be copied + mark only Non-empty cell?
                //       i.e., copy component and only add fr+1 and mark bt as FromM. do not define new values.
                if let Some(pre_components) = pre_wave_front_score.components_of_k_checked(k-1) {
                    let pre_m_component = &pre_components.m;
                    if pre_m_component.bt != BackTraceMarker::Empty {
                        unsafe {
                            (*new_components_of_k).d = Component {
                                fr: pre_m_component.fr + 1,
                                insertion_count: pre_m_component.insertion_count,
                                bt: BackTraceMarker::FromM,
                            };
                        }
                    }
                }
                // 2. Update I from previous M
                // TODO: Can be all components from previous wave front score be copied + mark only Non-empty cell?
                if let Some(pre_components) = pre_wave_front_score.components_of_k_checked(k+1) {
                    let pre_m_component = &pre_components.m;
                    if pre_m_component.bt != BackTraceMarker::Empty {
                        unsafe {
                            (*new_components_of_k).i = Component {
                                fr: pre_m_component.fr,
                                insertion_count: pre_m_component.insertion_count + 1,
                                bt: BackTraceMarker::FromM,
                            };
                        }
                    }
                }
            }
        }
        // (2) From score: s-e
        // Extended insertion or deletion
        if let Some(pre_score) = penalty.checked_sub(*gap_extend_penalty) {
            let pre_wave_front_score = &self.wave_front_scores[pre_score as usize];
            for index_of_k in 0..num_components {
                let k = index_of_k as i32 - max_k;
                let new_components_of_k = unsafe { new_components_ptr.add(index_of_k) };
                // 1. Update D from previous D
                if let Some(pre_components) = pre_wave_front_score.components_of_k_checked(k-1) {
                    let pre_d_component = &pre_components.d;
                    if pre_d_component.bt != BackTraceMarker::Empty {
                        unsafe {
                            // (If D is empty) OR (New FR is larger than previous values)
                            if (*new_components_of_k).d.bt == BackTraceMarker::Empty || (*new_components_of_k).d.fr < pre_d_component.fr + 1 {
                                (*new_components_of_k).d = Component {
                                    fr: pre_d_component.fr + 1,
                                    insertion_count: pre_d_component.insertion_count,
                                    bt: BackTraceMarker::FromD,
                                };
                            }
                        };
                    }
                }
                // 2. Update I from previous I
                if let Some(pre_components) = pre_wave_front_score.components_of_k_checked(k+1) {
                    let pre_i_component = &pre_components.i;
                    if pre_i_component.bt != BackTraceMarker::Empty {
                        unsafe {
                            if (*new_components_of_k).i.bt == BackTraceMarker::Empty || (*new_components_of_k).i.fr < pre_i_component.fr {
                                (*new_components_of_k).i = Component {
                                    fr: pre_i_component.fr,
                                    insertion_count: pre_i_component.insertion_count + 1,
                                    bt: BackTraceMarker::FromI,
                                };
                            };
                        }
                    }
                }
            }
        }
        // (3) From score: s-x
        // Substitution
        if let Some(pre_score) = penalty.checked_sub(*mismatch_penalty) {
            let pre_wave_front_score = &self.wave_front_scores[pre_score as usize];
            for index_of_k in 0..num_components {
                let k = index_of_k as i32 - max_k;
                let new_components_of_k = unsafe { new_components_ptr.add(index_of_k) };
                // 1. Update M from previous M
                let pre_component_index = (pre_wave_front_score.max_k + k) as usize;

                if let Some(pre_components) = pre_wave_front_score.components_by_k.get(pre_component_index) {
                    let pre_m_component = &pre_components.m;
                    // Update M
                    unsafe {
                        (*new_components_of_k).m = Component {
                            fr: pre_m_component.fr + 1,
                            insertion_count: pre_m_component.insertion_count,
                            bt: BackTraceMarker::FromM,
                        };
                    }
                }
            }
        }
        // TODO: Optimization
        for index_of_k in 0..num_components {
            let new_components_of_k = unsafe { new_components_ptr.add(index_of_k) };
            unsafe {
                // 2. Update M from current D
                if (*new_components_of_k).d.bt != BackTraceMarker::Empty && (
                    (*new_components_of_k).m.bt == BackTraceMarker::Empty
                    || (*new_components_of_k).d.fr >= (*new_components_of_k).m.fr
                ) {
                    (*new_components_of_k).m = Component {
                        fr: (*new_components_of_k).d.fr,
                        insertion_count: (*new_components_of_k).d.insertion_count,
                        bt: BackTraceMarker::FromD,
                    };
                }
                // 3. Update M from current I
                if (*new_components_of_k).i.bt != BackTraceMarker::Empty && (
                    (*new_components_of_k).m.bt == BackTraceMarker::Empty
                    || (*new_components_of_k).i.fr >= (*new_components_of_k).m.fr
                ) {
                    (*new_components_of_k).m = Component {
                        fr: (*new_components_of_k).i.fr,
                        insertion_count: (*new_components_of_k).i.insertion_count,
                        bt: BackTraceMarker::FromI,
                    };
                }
            }
        }
    }
}

impl WaveFrontScore {
    #[inline]
    fn add_first_components(&mut self, first_match_count: i32) {
        self.components_by_k = vec![Components::new_start_point(first_match_count)];
    }
    #[inline]
    fn extend_m_components_to_the_end<C: MatchCounter>(
        &mut self,
        tgt_seq: &[u8],
        qry_seq: &[u8],
    ) -> Option<i32> {
        for (components, k) in self.components_by_k.iter_mut().zip(-self.max_k..=self.max_k) {
            let m_component = &mut components.m;

            if m_component.bt != BackTraceMarker::Empty {
                // Extend & update
                let mut v = (m_component.fr - k) as usize; // query length to this component
                let mut h = m_component.fr as usize; // target length to this component
                let match_count = C::count_consecutive_match(qry_seq, tgt_seq, v, h);
                m_component.fr += match_count;
                // Check exit condition
                v += match_count as usize;
                h += match_count as usize;
                if h == tgt_seq.len() || v == qry_seq.len() {
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
