use core::panic;
use std::cmp::{min, max};
use std::collections::HashSet;
use std::{u8, usize};

use crate::alignment::anchor;

use super::{FmIndex, Operation, EmpKmer, Cutoff, Scores};
use super::dropout_wfa::{dropout_wf_align, WF};
use fm_index::BackwardSearchIndex;

struct AnchorGroup<'a> {
    ref_seq: &'a [u8],
    qry_seq: &'a [u8],
    kmer: usize,
    emp_kmer: &'a EmpKmer,
    scores: &'a Scores,
    cutoff: &'a Cutoff,
    anchors: Vec<Anchor>,
}
impl<'a> AnchorGroup<'a> {
    fn new(
        ref_seq: &'a [u8], qry_seq: &'a [u8], index: &FmIndex,
        kmer: usize, emp_kmer: &'a EmpKmer, scores: &'a Scores, cutoff: &'a Cutoff
    ) -> Option<Self> {
        let ref_len = ref_seq.len();
        let qry_len = qry_seq.len();
        let search_count = qry_len / kmer;
        let mut anchors_preset: Vec<Anchor> = Vec::new();
        let mut anchor_existence: Vec<bool> = Vec::with_capacity(search_count+1); // first value is buffer
        // (1) Generate Anchors Proto
        {
            let mut anchors_cache: Option<Vec<Anchor>> = None;
            for i in 0..search_count {
                let qry_position = i*kmer;
                let pattern = &qry_seq[qry_position..qry_position+kmer];
                let search = index.search_backward(pattern);
                let positions = search.locate();
                // ** Check Impeccable Extension **
                match anchors_cache {
                    Some(anchors) => {
                        if positions.len() == 0 {
                            anchors_preset.extend(anchors);
                            anchors_cache = None;
                        } else {
                            let mut current_anchors: Vec<Anchor> = Vec::with_capacity(positions.len());
                            let mut ie_positions: Vec<u64> = Vec::new();
                            for anchor in anchors {
                                let mut ie_check = false;
                                for position in &positions {
                                    // impeccable extension occurs
                                    if *position as usize == anchor.position.0 + anchor.size {
                                        ie_positions.push(*position);
                                        ie_check = true;
                                        break;
                                    }
                                }
                                // push anchor
                                if !ie_check {
                                    anchors_preset.push(anchor);
                                } else {
                                    current_anchors.push(anchor.impeccable_extension(kmer));
                                }
                            }
                            // if position is not ie position: add to current anchors
                            for position in positions {
                                if !ie_positions.contains(&position) {
                                    current_anchors.push(
                                        Anchor::new(position as usize, i*kmer, kmer)
                                    );
                                }
                            }
                            anchors_cache = Some(current_anchors);
                        }
                        anchor_existence.push(true);
                    },
                    None => {
                        if positions.len() != 0 {
                            anchors_cache = Some(positions.into_iter().map(|x| {
                                Anchor::new(x as usize, i*kmer, kmer)
                            }).collect());
                        }
                        anchor_existence.push(false);
                    },
                }
            }
            // push last anchors
            match anchors_cache {
                Some(anchors) => {
                    anchors_preset.extend(anchors);
                    anchor_existence.push(true);
                },
                None => {
                    anchor_existence.push(false);
                },
            }
        }
        // check anchor exist
        if !anchor_existence.iter().any(|x| *x) {
            return None
        }
        // (2) Calculate the EMP values
        anchors_preset.iter_mut().for_each(|anchor| {
            anchor.estimate(ref_len, qry_len, kmer, &anchor_existence, &emp_kmer);
        });
        // (3) evaluate raw anchors
        anchors_preset.iter_mut().for_each(|anchor| {
            if !anchor.is_valid_raw(cutoff) {
                anchor.to_dropped();
            }
        });
        // (4) Set up checkpoints
        Anchor::create_check_points(&mut anchors_preset, scores, cutoff);
        Some(
            Self {
                ref_seq: ref_seq,
                qry_seq: qry_seq,
                kmer: kmer,
                emp_kmer: emp_kmer,
                scores: scores,
                cutoff: cutoff,
                anchors: anchors_preset,
            }
        )
    }
    // fn alignment(&mut self) {
    //     // Hind Alignment
    //     for anchor in &mut self.anchors {
    //         match &anchor.state {
    //             AnchorState::Raw(_) => {
    //                 anchor.to_hind_part_done(self.ref_seq, self.qry_seq, &self.scores, &self.cutoff);
    //             },
    //             _ => {},
    //         }
    //     }
    //     // Fore Alignment
    //     for anchor in &mut self.anchors {
    //         match &anchor.state {
    //             AnchorState::HindDone(_) => {
    //                 anchor.to_both_part_done(self.ref_seq, self.qry_seq, &self.scores, &self.cutoff);
    //             },
    //             _ => {},
    //         }
    //     }
    // }
}

#[derive(Debug)]
struct Anchor {
    position: (usize, usize), // (ref, qry)
    size: usize,
    state: AlignmentState,
    check_points: (Vec<usize>, Vec<usize>), // index of anchors (fore, hind)
    connected: HashSet<usize>, // connected anchors index set for used as symbol
}

#[derive(Debug)]
enum AlignmentState {
    Empty,
    Estimated(EmpBlock, EmpBlock), // Fore, Hind
    Exact(Option<AlignmentBlock>, AlignmentBlock), // Fore, Hind
    Dropped,
}

#[derive(Debug)]
struct EmpBlock {
    penalty: usize,
    length: usize,
}
impl EmpBlock {
    fn new(penalty: usize, length: usize) -> Self {
        Self {
            penalty,
            length,
        }
    }
}

#[derive(Debug)]
enum AlignmentBlock {
    Own(Vec<Operation>, usize), // operations, penalty
    Ref(usize, usize),
}

impl AlignmentBlock {
    fn len(&self) {

    }
}

// impl AlignmentBlock {
//     fn aligned_length(&self) -> (usize, usize) {
//         let ins = self.operations.iter().filter(|&op| *op == Operation::Ins).count();
//         let del = self.operations.iter().filter(|&op| *op == Operation::Del).count();
//         let len = self.operations.len();
//         (len-del, len-ins)
//     }
//     fn clip_operation(&self, ref_len: usize, qry_len: usize) -> Operation {
//         let (ref_aligned_length, qry_aligned_length) = self.aligned_length();
//         let ref_left = ref_len-ref_aligned_length;
//         let qry_left = qry_len-qry_aligned_length;
//         if ref_left >= qry_left {
//             Operation::RefClip(ref_left-qry_left)
//         } else {
//             Operation::QryClip(qry_left-ref_left)
//         }
//     }
// }

impl Anchor {
    fn new(ref_pos: usize, qry_pos: usize, kmer: usize) -> Self {
        Self {
            position: (ref_pos, qry_pos),
            size: kmer,
            state:AlignmentState::Empty,
            check_points: (Vec::new(), Vec::new()),
            connected: HashSet::new(),
        }
    }
    fn impeccable_extension(mut self, kmer: usize) -> Self {
        self.size += kmer;
        self
    }
    fn estimate(&mut self, ref_len: usize, qry_len: usize, kmer: usize, anchor_existence: &Vec<bool>, emp_kmer: &EmpKmer) {
        let block_index = self.position.1 / kmer;
        // fore block
        let fore_emp_block = {
            let block_len = min(self.position.0, self.position.1);
            let quot = block_len / kmer;
            let mut odd_block_count: usize = 0;
            let mut even_block_count: usize = 0;
            let mut previous_block_is_odd = false;
            anchor_existence[(block_index-quot+1)..block_index+1].iter().rev().for_each(|exist| {
                if !*exist {
                    if previous_block_is_odd {
                        even_block_count += 1;
                        previous_block_is_odd = false;
                    } else {
                        odd_block_count += 1;
                        previous_block_is_odd = true;
                    }
                } else {
                    previous_block_is_odd = false;
                }
            });
            EmpBlock::new(
                odd_block_count*emp_kmer.odd + even_block_count*emp_kmer.even,
                block_len + odd_block_count + even_block_count
            )
        };
        // hind block
        let hind_emp_block = {
            let hind_block_index = block_index+(self.size/kmer);
            let ref_block_len = ref_len - (self.position.0 + self.size);
            let qry_block_len = qry_len - (self.position.1 + self.size);
            let block_len = min(ref_block_len, qry_block_len);
            let quot = block_len / kmer;
            let mut odd_block_count: usize = 0;
            let mut even_block_count: usize = 0;
            let mut previous_block_is_odd = false;
            anchor_existence[hind_block_index+1..hind_block_index+quot+1].iter().for_each(|exist| {
                if !*exist {
                    if previous_block_is_odd {
                        even_block_count += 1;
                        previous_block_is_odd = false;
                    } else {
                        odd_block_count += 1;
                        previous_block_is_odd = true;
                    }
                } else {
                    previous_block_is_odd = false;
                }
            });
            EmpBlock::new(
                odd_block_count*emp_kmer.odd + even_block_count*emp_kmer.even,
                block_len + odd_block_count + even_block_count
            )
        };
        self.state = AlignmentState::Estimated(fore_emp_block, hind_emp_block);
    }
    // query block stacked in order in anchors_preset
    // : high index is always the hind anchor
    fn can_be_connected(first: &Self, second: &Self, scores: &Scores, cutoff: &Cutoff) -> bool {
        let ref_gap = second.position.0 as i64 - first.position.0 as i64 - first.size as i64;
        let qry_gap = second.position.1 as i64 - first.position.1 as i64 - first.size as i64;
        if (ref_gap >= 0) && (qry_gap >= 0) {
            let mut penalty: usize = 0;
            let mut length: usize = 0;
            // fore
            if let AlignmentState::Estimated(emp_block, _) = &first.state {
                penalty += emp_block.penalty;
                length += emp_block.length;
            }
            // hind
            if let AlignmentState::Estimated(_, emp_block) = &second.state {
                penalty += emp_block.penalty;
                length += emp_block.length;
            }
            // middle
            length += max(ref_gap, qry_gap) as usize + first.size + second.size;
            let indel = (ref_gap - qry_gap).abs() as usize;
            if indel > 0 {
                penalty += scores.1 + indel*scores.2;
            }
            if (penalty as f64 / length as f64 <= cutoff.score_per_length) & (length >= cutoff.minimum_length) {
                true
            } else {
                false
            }
        } else {
            false
        }
    }
    fn extend_check_point_together(anchors: &mut Vec<Self>, first_index: usize, second_index: usize) {
        anchors[first_index].check_points.1.push(second_index);
        anchors[second_index].check_points.0.push(first_index);
    }
    fn both_estimated(anchor_1: &Self, anchor_2: &Self) -> bool {
        (match &anchor_1.state {
            AlignmentState::Estimated(_, _) => true,
            _ => false,
        }) && (match &anchor_2.state {
            AlignmentState::Estimated(_, _) => true,
            _ => false,
        })
    }
    fn create_check_points(anchors: &mut Vec<Self>, scores: &Scores, cutoff: &Cutoff) {
        let anchor_count = anchors.len();
        for index_1 in 0..anchor_count {
            for index_2 in index_1+1..anchor_count {
                if Self::both_estimated(&anchors[index_1], &anchors[index_2]) && Self::can_be_connected(&anchors[index_1], &anchors[index_2], &scores, &cutoff) {
                    Self::extend_check_point_together(anchors, index_1, index_2);
                }
            }
        }
    }
    fn estimated_to_hind_alignment(anchors: &mut Vec<Self>, current_index: usize, ref_seq: &[u8], qry_seq: &[u8], scores: &Scores, cutoff: &Cutoff) {
        // get refernce of current anchor
        let current_anchor = &mut anchors[current_index];
        let alignment_res = {
            let (p_other, l_other) = match &current_anchor.state {
                AlignmentState::Estimated(emp_block, _) => {
                    (emp_block.penalty, emp_block.length)
                },
                // FIXME: if not estimated -> just pass to next
                _ => panic!(""), // return
            };
            let panalty_spare = cutoff.score_per_length * (
                min(
                    ref_seq.len() - current_anchor.position.0 - current_anchor.size, qry_seq.len() - current_anchor.position.1 - current_anchor.size
                ) + current_anchor.size + l_other
            ) as f64 - p_other as f64;
             dropout_wf_align(&qry_seq[current_anchor.position.1+current_anchor.size..], &ref_seq[current_anchor.position.0+current_anchor.size..], scores, panalty_spare, cutoff.score_per_length)
        };
        match alignment_res {
            // Not dropped
            Ok((operations, penalty, wf)) => {
                // update state
                current_anchor.state = AlignmentState::Exact(
                    None,
                    AlignmentBlock::Own(operations, penalty),
                );
                // wf inheritant check
                for &check_point in &current_anchor.check_points.1 {
                    let test= check_point;
                }
            },
            // If dropped
            Err(wf) => {
                // TODO: WF inheritance algorithm
                current_anchor.to_dropped();
            },
        }
    }
    fn wf_inherit(&self, ) {

    }
    // fn penalty_and_length_of_side(&self, block_type: BlockType) -> (usize, usize) {
    //     match block_type {
    //         BlockType::Fore => {
    //             match &self.state {
    //                 AlignmentState::Estimated(emp_block, _) => {
    //                     (emp_block.penalty, emp_block.length)
    //                 },
    //                 // TODO: modify error msg
    //                 _ => panic!("alignment error")
    //             }
    //         },
    //         BlockType::Hind => {
    //             match &self.state {
    //                 AlignmentState::Estimated(_, emp_block) => {
    //                     (emp_block.penalty, emp_block.length)
    //                 },
    //                 AlignmentState::Exact(_, alignment_block) => {
    //                     (alignment_block.1, alignment_block.0.len())
    //                 },
    //                 // TODO: modify error msg
    //                 _ => panic!("alignment error")
    //             }
    //         },
    //     }
    // }
    // fn update_hind(&mut self, operations: Vec<Operation>, penalty: usize) {
    //     self.state = AnchorState::HindDone(Some(
    //         AlignmentBlock {
    //             operations,
    //             penalty,
    //         },
    //     ));
    // }
    // fn update_fore(&mut self, operations: Vec<Operation>, penalty: usize) {
    //     if let AnchorState::HindDone(block_option) = &mut self.state {
    //         let alignment_block = match block_option.take() {
    //             Some(v) => v,
    //             None => panic!("Hind block cannot be None"),
    //         };
    //         self.state = AnchorState::BothDone((
    //             AlignmentBlock {
    //                 operations,
    //                 penalty,
    //             },
    //             alignment_block,
    //         ));
    //     }
    // }
    // fn to_hind_part_done(&mut self, ref_seq: &[u8], qry_seq: &[u8], scores: &Scores, cutoff: &Cutoff) {
    //     let (p_other, l_other) = self.penalty_and_length_of_side(BlockType::Fore);
    //     let panalty_spare = cutoff.score_per_length * (
    //         min(
    //             ref_seq.len() - self.position.0 - self.size, qry_seq.len() - self.position.1 - self.size
    //         ) + self.size + l_other
    //     ) as f64 - p_other as f64;
    //     let alignment_res = dropout_wf_align(&qry_seq[self.position.1+self.size..], &ref_seq[self.position.0+self.size..], scores, panalty_spare, cutoff.score_per_length);
    //     match alignment_res {
    //         Ok((operations, penalty)) => {
    //             self.update_hind(operations, penalty);
    //         },
    //         Err(wf) => {
    //             // TODO: WF inheritance algorithm
    //             self.to_dropped();
    //         },
    //     }
    // }
    // fn to_both_part_done(&mut self, ref_seq: &[u8], qry_seq: &[u8], scores: &Scores, cutoff: &Cutoff) {
    //     let (p_other, l_other) = self.penalty_and_length_of_side(BlockType::Hind);
    //     let panalty_spare = cutoff.score_per_length * (min(self.position.0, self.position.1) + self.size + l_other) as f64 - p_other as f64;
    //     let ref_seq: Vec<u8> = ref_seq[0..self.position.0].iter().rev().map(|x| *x).collect();
    //     let qry_seq: Vec<u8> = qry_seq[0..self.position.1].iter().rev().map(|x| *x).collect();
    //     let alignment_res = dropout_wf_align(&qry_seq, &ref_seq, scores, panalty_spare, cutoff.score_per_length);
    //     match alignment_res {
    //         Ok((mut operations, penalty)) => {
    //             operations.reverse();
    //             self.update_fore(operations, penalty);
    //         },
    //         Err(wf) => {
    //             // TODO: WF inheritance algorithm
    //             self.to_dropped();
    //         },
    //     }
    // }
    fn to_dropped(&mut self) {
        self.state = AlignmentState::Dropped;
    }
    fn is_valid_raw(&self, cutoff: &Cutoff) -> bool{
        if let AlignmentState::Estimated(emp_block_1, emp_block_2) = &self.state {
            let length = emp_block_1.length + emp_block_2.length + self.size;
            if length >= cutoff.minimum_length && (emp_block_1.penalty + emp_block_2.penalty) as f64/length as f64 <= cutoff.score_per_length {
                true
            } else {
                false
            }
        } else {
            // TODO: error msg
            panic!("");
        }
    }
    // fn evaluate(self, ref_len: usize, qry_len: usize, cutoff: &Cutoff) -> Option<(Vec<Operation>, usize)>{
    //     let (fore_block, hind_block) = match self.state {
    //         AnchorState::BothDone(v) => v,
    //         _ => panic!("Only block with both aligned can be evaluated."),
    //     };
    //     let length = fore_block.operations.len() + hind_block.operations.len() + self.size;
    //     let penalty = fore_block.penalty + hind_block.penalty;
    //     #[cfg(test)]
    //     println!("len:{}, pen:{}", length, penalty);
    //     if (length >= cutoff.minimum_length) && (penalty as f64/length as f64 <= cutoff.score_per_length) {
    //         let mut operations = Vec::with_capacity(length+2);
    //         operations.push(fore_block.clip_operation(self.position.0, self.position.1));
    //         operations.extend(fore_block.operations);
    //         operations.extend(vec![Operation::Match; self.size]);
    //         let hind_clip = hind_block.clip_operation(ref_len-self.position.0-self.size, qry_len-self.position.1-self.size);
    //         operations.extend(hind_block.operations);
    //         operations.push(hind_clip);
    //         Some((operations, penalty))
    //     } else {
    //         None
    //     }
    // }
}

// #[derive(Debug)]
// enum AnchorState {
//     Proto,
//     Raw((EmpBlock, EmpBlock)), // Fore, Hind
//     HindDone(Option<AlignmentBlock>), // Hind
//     BothDone((AlignmentBlock, AlignmentBlock)), // Fore, Hind
//     Dropped,
// }

// #[derive(Debug)]
// enum BlockType {
//     Fore,
//     Hind,
// }

/*
#[cfg(test)]
mod tests {
    use crate::alignment::test_data;
    use super::*;

    fn print_anchor_group() {
        let test_data = test_data::get_test_data();
        let seqs = test_data[1].clone();
        let ref_seq = seqs.0.as_bytes();
        let qry_seq = seqs.1.as_bytes();
        let index = super::super::Reference::fmindex(&ref_seq);
        let aligner = super::super::tests::test_aligner();
        let anchor_group = AnchorGroup::new(&ref_seq, &qry_seq, &index, aligner.kmer, &aligner.emp_kmer, &aligner.scores, &aligner.cutoff).unwrap();
        println!("{:?}", anchor_group.anchors);
    }

    #[test]
    fn print_aligned_anchor_group() {
        let test_data = test_data::get_test_data();
        let seqs = test_data[1].clone();
        let ref_seq = seqs.0.as_bytes();
        let qry_seq = seqs.1.as_bytes();
        let index = super::super::Reference::fmindex(&ref_seq);
        let aligner = super::super::tests::test_aligner();
        let mut anchor_group = AnchorGroup::new(&ref_seq, &qry_seq, &index, aligner.kmer, &aligner.emp_kmer, &aligner.scores, &aligner.cutoff).unwrap();
        anchor_group.alignment();
        for anchor in anchor_group.anchors {
            let res = anchor.evaluate(anchor_group.ref_seq.len(), anchor_group.qry_seq.len(), anchor_group.cutoff);
            println!("{:?}", res);
        }
    }
}
*/