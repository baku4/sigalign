use crate::{SequenceLength, OperationLength};

/// Aligned seqeuence block  
/// Vector of (type of operation, and its length)
pub type AlignedBlock = Vec<(Operation, OperationLength)>;
// ReverseIndex: (index from end, count offset, ins count, del count)
pub type ReverseIndex = (usize, OperationLength, OperationLength, OperationLength);

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Operation {
    Match,
    Subst,
    Ins,
    Del,
}

#[derive(Debug, Clone, Copy)]
pub enum Clip {
    None,
    Qry(SequenceLength),
    Ref(SequenceLength),
}

impl Clip {
    #[inline]
    pub fn new(
        ref_len: SequenceLength,
        qry_len: SequenceLength,
        ref_aligned_length: SequenceLength,
        qry_aligned_length: SequenceLength
    ) -> Self {
        let ref_left = ref_len - ref_aligned_length;
        let qry_left = qry_len - qry_aligned_length;
        if ref_left == qry_left {
            Clip::None
        } else if ref_left > qry_left {
            Clip::Ref(ref_left - qry_left)
        } else {
            Clip::Qry(qry_left - ref_left)
        }
    }
}

#[inline]
pub fn get_reverse_index_from_own(reversed_cigar: &AlignedBlock) -> ReverseIndex { 
    let mut ins: u32 = 0;
    let mut del: u32 = 0;
    reversed_cigar.iter().for_each(|(op, count)| {
        match op {
            Operation::Ins => { ins += count; },
            Operation::Del => { del += count; },
            _ => (),
        }
    });
    (reversed_cigar.len(), reversed_cigar[reversed_cigar.len()-1].1, ins, del)
}
#[inline]
pub fn get_reverse_index_from_ref(reversed_cigar: &AlignedBlock, length: &usize) -> ReverseIndex {
    let mut ins: u32 = 0;
    let mut del: u32 = 0;
    let mut index_from_end: usize = 1;
    let mut count_offset: u32 = *length as u32;
    for &(op, count) in reversed_cigar {
        if count_offset <= count {
            match op {
                Operation::Ins => { ins += count_offset; },
                Operation::Del => { del += count_offset; },
                _ => (),
            }
            break;
        } else {
            match op {
                Operation::Ins => { ins += count; },
                Operation::Del => { del += count; },
                _ => (),
            }
            count_offset -= count;
            index_from_end += 1;
        }
    }
    (index_from_end, count_offset, ins, del)
}
#[inline]
pub fn new_cigar_from_ref(ref_cigar: &AlignedBlock, reverse_index: &ReverseIndex) -> AlignedBlock {
    let mut new_cigar: AlignedBlock = ref_cigar[ref_cigar.len()-reverse_index.0..].to_vec();
    new_cigar[0].1 -= reverse_index.1;
    new_cigar
}