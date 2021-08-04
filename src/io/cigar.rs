use crate::{SequenceLength, OperationLength};

pub type Cigar = Vec<(Operation, OperationLength)>;
// ReverseIndex: (index from end, count offset, ins count, del count)
pub type ReverseIndex = (usize, OperationLength, OperationLength, OperationLength);

#[derive(Debug, Clone, Copy)]
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

#[inline]
pub fn get_reverse_index_from_own(reversed_cigar: &Cigar) -> ReverseIndex { 
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
pub fn get_reverse_index_from_ref(reversed_cigar: &Cigar, length: &usize) -> ReverseIndex {
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
pub fn new_cigar_from_ref(ref_cigar: &Cigar, reverse_index: &ReverseIndex) -> Cigar {
    let mut new_cigar: Cigar = ref_cigar[ref_cigar.len()-reverse_index.0..].to_vec();
    new_cigar[0].1 -= reverse_index.1;
    new_cigar
}