// TODO: apply SIMD
pub trait MatchCounter {
    fn count_consecutive_match(
        qry_seq: &[u8],
        tgt_seq: &[u8],
        qry_start_index: usize,
        tgt_start_index: usize,
    ) -> i32;
}

pub struct ForwardMatchCounter;
impl MatchCounter for ForwardMatchCounter {
    #[inline(always)]
    fn count_consecutive_match(
        qry_seq: &[u8],
        tgt_seq: &[u8],
        qry_start_index: usize,
        tgt_start_index: usize,
    ) -> i32 {
        let mut match_count: i32 = 0; // Same as "fr to add"
        for (v1, v2) in qry_seq[qry_start_index..].iter().zip(tgt_seq[tgt_start_index..].iter()) {
            if *v1 == *v2 {
                match_count += 1;
            } else {
                return match_count
            }
        }
        match_count
    }
}
pub struct ReverseMatchCounter;
impl MatchCounter for ReverseMatchCounter {
    #[inline(always)]
    fn count_consecutive_match(
        qry_seq: &[u8],
        tgt_seq: &[u8],
        qry_start_index: usize,
        tgt_start_index: usize,
    ) -> i32 {
        let mut match_count: i32 = 0; // Same as "fr to add"
        for (v1, v2) in qry_seq[..qry_seq.len()-qry_start_index].iter().rev().zip(tgt_seq[..tgt_seq.len()-tgt_start_index].iter().rev()) {
            if *v1 == *v2 {
                match_count += 1;
            } else {
                return match_count
            }
        }
        match_count
    }
}
