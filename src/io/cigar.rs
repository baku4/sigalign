use std::slice::Iter;

pub type Cigar = Vec<(Operation, u32)>;
pub type IterCigar<'a> = &'a Iter<'a, (Operation, u32)>;
pub type ReverseIndex = (usize, u32);

#[derive(Debug, Clone, Copy)]
pub enum Operation {
    Match,
    Subst,
    Ins,
    Del,
    RefClip,
    QryClip,
}

#[inline]
pub fn get_reverse_index(ref_cigar: &Cigar, length: &usize) -> ReverseIndex { // (reverse_index, count_offset)
    let mut index_from_end: usize = 0;
    let mut count_offset: u32 = *length as u32;
    for &(_, op_count) in ref_cigar.iter().rev() {
        if count_offset < op_count {
            break;
        } else {
            count_offset -= op_count;
            index_from_end += 1;
        }
    }
    (index_from_end, count_offset)
}
#[inline]
pub fn new_cigar_from_ref(ref_cigar: &Cigar, reverse_index: &ReverseIndex) -> Cigar {
    let mut new_cigar: Cigar = ref_cigar[ref_cigar.len()-reverse_index.0..].to_vec();
    new_cigar[0].1 -= reverse_index.1;
    new_cigar
}