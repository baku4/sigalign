#[inline(always)]
pub fn count_forward_2(
    ref_seq: &[u8],
    qry_seq: &[u8],
    v: usize,
    h: usize,
) -> i32 {
    qry_seq[v..]
        .iter()
        .zip(ref_seq[h..].iter())
        .position(|(v1, v2)| v1 != v2)
        .map(|pos| pos as i32)
        .unwrap_or((qry_seq.len() - v) as i32)
}
#[inline(always)]
pub fn count_backward_2(
    ref_seq: &[u8],
    qry_seq: &[u8],
    v: usize,
    h: usize,
) -> i32 {
    qry_seq[..qry_seq.len() - v]
        .iter()
        .rev()
        .zip(ref_seq[..ref_seq.len() - h].iter().rev())
        .position(|(v1, v2)| v1 != v2)
        .map(|pos| pos as i32)
        .unwrap_or((qry_seq.len() - v) as i32)
}
