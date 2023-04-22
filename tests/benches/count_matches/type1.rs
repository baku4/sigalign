#[inline(always)]
pub fn count_forward_1(
    ref_seq: &[u8],
    qry_seq: &[u8],
    v: usize,
    h: usize,
) -> i32 {
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
#[inline(always)]
pub fn count_backward_1(
    ref_seq: &[u8],
    qry_seq: &[u8],
    v: usize,
    h: usize,
) -> i32 {
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