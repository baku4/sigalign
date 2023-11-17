// TODO: apply SIMD
pub trait MatchCounter {
    fn count_consecutive_match(
        tgt_seq: &[u8],
        qry_seq: &[u8],
        v: usize,
        h: usize,
    ) -> i32;
}

pub struct ForwardMatchCounter;
impl MatchCounter for ForwardMatchCounter {
    #[inline(always)]
    fn count_consecutive_match(
        tgt_seq: &[u8],
        qry_seq: &[u8],
        v: usize,
        h: usize,
    ) -> i32 {
        let mut fr_to_add: i32 = 0;
        for (v1, v2) in qry_seq[v..].iter().zip(tgt_seq[h..].iter()) {
            if *v1 == *v2 {
                fr_to_add += 1;
            } else {
                return fr_to_add
            }
        }
        fr_to_add
    }
}
pub struct ReverseMatchCounter;
impl MatchCounter for ReverseMatchCounter {
    #[inline(always)]
    fn count_consecutive_match(
        tgt_seq: &[u8],
        qry_seq: &[u8],
        v: usize,
        h: usize,
    ) -> i32 {
        let mut fr_to_add: i32 = 0;
        for (v1, v2) in qry_seq[..qry_seq.len()-v].iter().rev().zip(tgt_seq[..tgt_seq.len()-h].iter().rev()) {
            if *v1 == *v2 {
                fr_to_add += 1;
            } else {
                return fr_to_add
            }
        }
        fr_to_add
    }
}
