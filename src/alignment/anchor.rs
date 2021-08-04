use crate::{SequenceLength, Penalty};
use crate::io::cigar::{
    Cigar, Operation, Clip, ReverseIndex,
    get_reverse_index_from_own, get_reverse_index_from_ref,
};
use super::{Cutoff, Penalties, BlockPenalty, FmIndex, Alignment, AlignmentResult};
use super::dwfa::{
    WaveFront, AnchorsToPassCheck, CigarReference, BacktraceResult,
    dropout_wf_align, dropout_wf_backtrace
};

use core::panic;
use std::cmp::{min, max};
use std::collections::{HashMap, HashSet};
use std::iter::FromIterator;

/// Anchor Group
pub struct AnchorGroup<'a> {
    ref_seq: &'a [u8],
    qry_seq: &'a [u8],
    penalties: &'a Penalties,
    cutoff: &'a Cutoff,
    anchors: Vec<Anchor>,
}
impl<'a> AnchorGroup<'a> {
    pub fn new(
        ref_seq: &'a [u8], qry_seq: &'a [u8], index: &FmIndex,
        kmer: usize, block_penalty: &'a BlockPenalty,
        penalties: &'a Penalties, cutoff: &'a Cutoff
    ) -> Option<Self> {
        let ref_len = ref_seq.len();
        let qry_len = qry_seq.len();
        let search_count = qry_len / kmer;
        // TODO: with cap
        let mut anchors_preset: Vec<Anchor> = Vec::new();
        let mut anchor_existence: Vec<bool> = Vec::with_capacity(search_count+1); // first value is buffer
        // (1) Generate Anchors Preset
        {
            let mut anchors_cache: Option<Vec<Anchor>> = None;
            for i in 0..search_count {
                let qry_position = i*kmer;
                let pattern = &qry_seq[qry_position..qry_position+kmer];
                let positions = index.locate_with_klt(pattern);
                // Check Impeccable Extension
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
            // check anchor exist
            if !anchor_existence.iter().any(|x| *x) {
                return None
            }
        }
        // (2) Estimate alignment from anchor preset
        anchors_preset.iter_mut().for_each(|anchor| {
            anchor.estimate_preset(ref_len, qry_len, kmer, &anchor_existence, &block_penalty);
            // if can't satisfy the cutoff -> drop
            if !anchor.estimated_state_is_valid(cutoff) {
                anchor.to_dropped();
            }
        });
        // (3) Set up checkpoints
        Anchor::create_check_points(&mut anchors_preset, penalties, cutoff);
        Some(
            Self {
                ref_seq: ref_seq,
                qry_seq: qry_seq,
                penalties,
                cutoff: cutoff,
                anchors: anchors_preset,
            }
        )
    }
    pub fn alignment(&mut self, using_cached_wf: bool) {
        // (1) alignment hind
        for idx in 0..self.anchors.len() {
            Anchor::alignment(
                &mut self.anchors, idx,
                self.ref_seq, self.qry_seq, self.penalties, self.cutoff,
                BlockType::Hind,
                using_cached_wf
            );
        }
        // (2) alignment fore
        // TODO: not use new vector
        let reversed_ref_seq: Vec<u8> = self.ref_seq.iter().rev().map(|x| *x).collect();
        let reversed_qry_seq: Vec<u8> = self.qry_seq.iter().rev().map(|x| *x).collect();
        for idx in (0..self.anchors.len()).rev() {
            Anchor::alignment(
                &mut self.anchors, idx,
                &reversed_ref_seq, &reversed_qry_seq, self.penalties, self.cutoff,
                BlockType::Fore,
                using_cached_wf
            );
        };
    }
    pub fn get_result(&mut self, get_minimum_penalty: bool) -> AlignmentResult {
        // (3) evaluate
        let anchors_of_minimum_penalty = if get_minimum_penalty {
            // TODO: first anchor can be evalauted only one time?
            let (mut minimum_penalty, _) = self.anchors[0].get_penalty_and_length();
            let mut anchors_of_minimum_penalty: HashSet<usize> = HashSet::new();
            for (anchor_index, anchor) in self.anchors.iter_mut().enumerate() {
                let (penalty, length) = anchor.get_penalty_and_length();
                if !Anchor::exact_alignment_is_valid(penalty, length, &self.cutoff) {
                    anchor.to_dropped();
                } else {
                    if penalty < minimum_penalty {
                        minimum_penalty = penalty;
                        anchors_of_minimum_penalty = HashSet::from_iter(vec![anchor_index]);
                    } else if penalty == minimum_penalty {
                        anchors_of_minimum_penalty.insert(anchor_index);
                    }
                }
            }
            Some(anchors_of_minimum_penalty)
        } else {
            for anchor in self.anchors.iter_mut() {
                let (penalty, length) = anchor.get_penalty_and_length();
                if !Anchor::exact_alignment_is_valid(penalty, length, &self.cutoff) {
                    anchor.to_dropped();
                };
            };
            None
        };
        // (4) get unique anchors
        let unqiue_anchors_index = Anchor::get_unique_symbols(&self.anchors, anchors_of_minimum_penalty);
        // (5) get cigar & penalty
        let ref_len = self.ref_seq.len();
        let qry_len = self.qry_seq.len();
        unqiue_anchors_index.into_iter().map(|anchor_index| {
            Anchor::get_alignment(&self.anchors, anchor_index, ref_len, qry_len)
        }).collect()
    }
}

/// Anchor
#[derive(Debug)]
pub struct Anchor {
    /// Positions of anchor
    /// (position of reference, position of qry)
    position: (SequenceLength, SequenceLength),
    /// Size of anchor
    size: SequenceLength,
    /// Alignment state of anchor
    state: AlignmentState,
    /// Index of other anchors to check on WF inheritance & backtrace.
    /// (fore, hind)
    check_points: (Vec<usize>, Vec<usize>),
    /// Cache for inherited WF
    wf_cache: Option<WaveFront>,
    /// Connected anchors index set for used as anchor's symbol
    connected: HashSet<usize>,
}

/// State of alignment
#[derive(Debug)]
pub enum AlignmentState {
    /// 1st state
    /// fore and hind alignments are empty
    Preset,
    /// 2nd state
    /// filled with blocks in the EMP state
    Estimated(EstAlign, EstAlign), // Fore, Hind
    /// 3rd, 4th state
    /// aligned exactly with `dropout wfa`
    Exact(Option<ExactAlign>, ExactAlign), // Fore, Hind
    /// Cutoff is not satisfied when aligned from anchor
    Dropped,
}

/// Alignment assumed when EMP state from anchor
#[derive(Debug)]
pub struct EstAlign {
    penalty: Penalty,
    length: SequenceLength,
}
impl EstAlign {
    #[inline]
    fn new(penalty: Penalty, length: SequenceLength) -> Self {
        Self {
            penalty,
            length,
        }
    }
}

/// One-way semi-global alignment from anchor
#[derive(Debug)]
pub enum ExactAlign {
    /// Having an operations.
    Own(BacktraceResult), 
    /// Referring to the operation of another anchor.
    /// (index of connected anchor, length, penalty)
    Ref(usize, CigarReference),
}
impl ExactAlign {
    #[inline]
    fn length_and_penalty(&self) -> (SequenceLength, Penalty) {
        match self {
            &Self::Own((_, len, p)) => {
                (len, p)
            },
            &Self::Ref(_, v) => {
                v
            }
        }
    }
}

impl Anchor {
    /*
    INITIALIZATION
    */
    /// New anchor in Empty state
    #[inline]
    fn new(ref_pos: usize, qry_pos: usize, kmer: SequenceLength) -> Self {
        Self {
            position: (ref_pos, qry_pos),
            size: kmer,
            state: AlignmentState::Preset,
            check_points: (Vec::new(), Vec::new()),
            wf_cache: None,
            connected: HashSet::new(),
        }
    }
    /// When the anchor is completely connected, both anchors are treated as one anchor.
    #[inline]
    fn impeccable_extension(mut self, kmer: SequenceLength) -> Self {
        self.size += kmer;
        self
    }
    /// Empty anchor to estimated state
    #[inline]
    fn estimate_preset(&mut self, ref_len: usize, qry_len: usize, kmer: SequenceLength, anchor_existence: &Vec<bool>, emp_kmer: &BlockPenalty) {
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
            EstAlign::new(
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
            EstAlign::new(
                odd_block_count*emp_kmer.odd + even_block_count*emp_kmer.even,
                block_len + odd_block_count + even_block_count
            )
        };
        self.state = AlignmentState::Estimated(fore_emp_block, hind_emp_block);
    }
    #[inline]
    fn estimated_state_is_valid(&self, cutoff: &Cutoff) -> bool {
        if let AlignmentState::Estimated(emp_block_1, emp_block_2) = &self.state {
            let length = emp_block_1.length + emp_block_2.length + self.size;
            if length >= cutoff.minimum_length && (emp_block_1.penalty + emp_block_2.penalty) as f64/length as f64 <= cutoff.score_per_length {
                true
            } else {
                false
            }
        } else {
            panic!("Anchor is not in EMP state.");
        }
    }
    /*
    CHECK POINT
    */
    // query block stacked in order in anchors_preset
    // : high index is always the hind anchor
    #[inline]
    fn can_be_connected(first: &Self, second: &Self, penalties: &Penalties, cutoff: &Cutoff) -> bool {
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
                penalty += penalties.o + indel*penalties.e;
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
    #[inline]
    fn extend_each_check_points(anchors: &mut Vec<Self>, first_index: usize, second_index: usize) {
        anchors[first_index].check_points.1.push(second_index);
        anchors[second_index].check_points.0.push(first_index);
    }
    #[inline]
    fn is_both_estimated(anchor_1: &Self, anchor_2: &Self) -> bool {
        (match &anchor_1.state {
            AlignmentState::Estimated(_, _) => true,
            _ => false,
        }) && (match &anchor_2.state {
            AlignmentState::Estimated(_, _) => true,
            _ => false,
        })
    }
    #[inline]
    fn create_check_points(anchors: &mut Vec<Self>, penalties: &Penalties, cutoff: &Cutoff) {
        let anchor_count = anchors.len();
        for index_1 in 0..anchor_count {
            for index_2 in index_1+1..anchor_count {
                if Self::is_both_estimated(&anchors[index_1], &anchors[index_2]) && Self::can_be_connected(&anchors[index_1], &anchors[index_2], &penalties, &cutoff) {
                    Self::extend_each_check_points(anchors, index_1, index_2);
                }
            }
        }
    }
    #[inline]
    fn get_check_points_for_backtrace(anchors: &Vec<Self>, current_index: usize, block_type: BlockType) -> AnchorsToPassCheck {
        let current_anchor = &anchors[current_index];
        match block_type {
            BlockType::Fore => {
                let check_points = &current_anchor.check_points.0;
                let mut backtrace_check_points: AnchorsToPassCheck = Vec::with_capacity(check_points.len());
                check_points.into_iter().for_each(|&anchor_index| {
                    let anchor = &anchors[anchor_index];
                    let ref_gap = (current_anchor.position.0 - anchor.position.0) as i32;
                    let qry_gap = (current_anchor.position.1 - anchor.position.1) as i32;
                    backtrace_check_points.push((anchor_index, anchor.size as i32, ref_gap-qry_gap, ref_gap));
                });
                backtrace_check_points
            },
            BlockType::Hind => {
                let check_points = &current_anchor.check_points.1;
                let mut backtrace_check_points: AnchorsToPassCheck = Vec::with_capacity(check_points.len());
                check_points.into_iter().for_each(|&anchor_index| {
                    let anchor = &anchors[anchor_index];
                    let ref_gap = (anchor.position.0 + anchor.size - current_anchor.position.0 - current_anchor.size) as i32;
                    let qry_gap = (anchor.position.1 + anchor.size - current_anchor.position.1 - current_anchor.size) as i32;
                    backtrace_check_points.push((anchor_index, anchor.size as i32, ref_gap-qry_gap, ref_gap));
                });
                backtrace_check_points
            },
        }
    }
    /* TODO: write inherit function
    fn wf_inheritance_check_points(anchors: &Vec<Self>, current_index: usize, ref_seq: &[u8], qry_seq: &[u8], block_type: BlockType) -> ChkpInherit {
        let current_anchor = &anchors[current_index];
        match block_type {
            BlockType::Fore => {
                let check_points = &current_anchor.check_points.0;
                let mut inheritance_check_points: ChkpInherit = HashMap::with_capacity(check_points.len());
                check_points.iter().for_each(|&anchor_index| {
                    let anchor = &anchors[anchor_index];
                    if let AlignmentState::Exact(None, _) = &anchor.state {
                        let (ref_pos, qry_pos) = anchor.position;
                        let mut ext_count: usize = 1;
                        loop {
                            if let Some(ref_char) = ref_seq.get(ref_pos - ext_count) {
                                if let Some(qry_char) = qry_seq.get(qry_pos - ext_count) {
                                    if *ref_char == *qry_char {
                                        ext_count += 1
                                    } else {
                                        break;
                                    }
                                } else {
                                    break;
                                }
                            } else {
                                break;
                            }
                        };
                        let ref_gap = (current_anchor.position.0 - anchor.position.0) as i32;
                        let qry_gap = (current_anchor.position.1 - anchor.position.1) as i32;
                        inheritance_check_points.insert(anchor_index, (anchor.size, ref_gap-qry_gap, ref_gap, ref_gap+ext_count as i32-1));
                    };
                });
                inheritance_check_points
            },
            BlockType::Hind => {
                let check_points = &current_anchor.check_points.1;
                let mut inheritance_check_points: ChkpInherit = HashMap::with_capacity(check_points.len());
                check_points.iter().for_each(|&anchor_index| {
                    let anchor = &anchors[anchor_index];
                    if let AlignmentState::Estimated(_, _) = &anchor.state {
                        let (ref_pos, qry_pos) = anchor.position;
                        let mut ext_count: usize = 1;
                        loop {
                            if let Some(ref_char) = ref_seq.get(ref_pos + anchor.size + ext_count) {
                                if let Some(qry_char) = qry_seq.get(qry_pos + anchor.size +  ext_count) {
                                    if *ref_char == *qry_char {
                                        ext_count += 1
                                    } else {
                                        break;
                                    }
                                } else {
                                    break;
                                }
                            } else {
                                break;
                            }
                        };
                        let ref_gap = (anchor.position.0 + anchor.size - current_anchor.position.0 - current_anchor.size) as i32;
                        let qry_gap = (anchor.position.1 + anchor.size - current_anchor.position.1 - current_anchor.size) as i32;
                        inheritance_check_points.insert(anchor_index, (anchor.size, ref_gap-qry_gap, ref_gap, ref_gap+ext_count as i32-1));
                    };
                });
                inheritance_check_points
            },
        }
    }
    */
    /*
    ALIGNMENT
    */
    #[inline]
    fn alignment(
        anchors: &mut Vec<Self>, current_anchor_index: usize,
        ref_seq: &[u8], qry_seq: &[u8],
        penalties: &Penalties, cutoff: &Cutoff,
        block_type: BlockType, using_cached_wf: bool,
    ) {
        #[cfg(test)]
        {
            println!("current index: {:?} / pos: {:?}", current_anchor_index, anchors[current_anchor_index].position);
        }
        // (1) DWFA
        let dwfa_res = {
            // get refernce of current anchor
            let current_anchor = &mut anchors[current_anchor_index];
            let (l_opp, p_opp) = match block_type {
                BlockType::Hind => {
                    match &current_anchor.state {
                        AlignmentState::Estimated(est_align, _) => {
                            (est_align.penalty, est_align.length)
                        },
                        // if not estimated(hind block is already done) or dropped
                        // -> just pass to next
                        _ => return,
                    }
                },
                BlockType::Fore => {
                    match &current_anchor.state {
                        AlignmentState::Exact(fore, hind) => {
                            match fore {
                                None => {
                                    hind.length_and_penalty()
                                },
                                _ => return,
                            }
                        },
                        _ => return,
                    }
                },
            };
            let penalty_spare = match block_type {
                BlockType::Hind => {
                    (
                        penalties.e as f64 * cutoff.score_per_length * (
                            l_opp + current_anchor.size + min(
                                ref_seq.len() - current_anchor.position.0 - current_anchor.size,
                                qry_seq.len() - current_anchor.position.1 - current_anchor.size,
                            )
                        ) as f64
                        - (penalties.e * p_opp) as f64
                        - penalties.o as f64 * cutoff.score_per_length
                    ) / (penalties.e - 1) as f64
                },
                BlockType::Fore => {
                    (
                        penalties.e as f64 * cutoff.score_per_length * (
                            l_opp + current_anchor.size + min(
                                current_anchor.position.0,
                                current_anchor.position.1,
                            )
                        ) as f64
                        - ( penalties.e * p_opp) as f64
                        - penalties.o as f64 * cutoff.score_per_length
                    ) / (penalties.e - 1) as f64
                }
            }.ceil() as usize;
            // Get cached wf
            // if current anchor has cached wf -> continue with cached wf
            let wf_cache = current_anchor.wf_cache.take();
            #[cfg(test)]
            {
                if let Some(v) = &wf_cache {
                    println!("using inherited wf:\n{:?}\n{:?}, ", v[0], v[1]);
                };
            }
            // DWFA
            match block_type {
                BlockType::Hind => {
                    match wf_cache {
                        // TODO: inherit
                        /*
                        Some(wf) => {
                            dropout_inherited_wf_align(wf, &qry_seq[current_anchor.position.1+current_anchor.size..], &ref_seq[current_anchor.position.0+current_anchor.size..], penalties, panalty_spare, cutoff.score_per_length)
                        },
                        */
                        _ => {
                            dropout_wf_align(
                                &qry_seq[current_anchor.position.1+current_anchor.size..],
                                &ref_seq[current_anchor.position.0+current_anchor.size..],
                                penalty_spare,
                                penalties
                            )
                        },
                    }
                },
                BlockType::Fore => {
                    // sequence must be reversed !
                    match wf_cache {
                        // TODO: inherit
                        /*
                        Some(wf) => {
                            dropout_inherited_wf_align(wf, &qry_seq[qry_seq.len()-current_anchor.position.1..], &ref_seq[ref_seq.len()-current_anchor.position.0..], penalties, penalty_spare, cutoff.score_per_length)
                        },
                        */
                        _ => {
                            dropout_wf_align(
                                &qry_seq[qry_seq.len()-current_anchor.position.1..],
                                &ref_seq[ref_seq.len()-current_anchor.position.0..],
                                penalty_spare,
                                penalties
                            )
                        },
                    }
                },
            }
        };
        // (2) BACKTRACE
        match dwfa_res {
            /*
            CASE 1: wf not dropped
            */
            Ok((mut wf, current_anchor_score, last_k)) => {
                // (1) get check points for backtrace
                let anchors_to_chk: AnchorsToPassCheck = Self::get_check_points_for_backtrace(
                    anchors, current_anchor_index, block_type.clone()
                );
                // (2) bactrace
                let (alignment_res, connected_backtraces) = dropout_wf_backtrace(
                    &wf, penalties, current_anchor_score, last_k, &anchors_to_chk
                );
                // (3) get valid anchor index
                let connected_anchor_indices: HashSet<usize> = HashSet::from_iter(
                    connected_backtraces.keys().map(|x| *x)
                );
                // (4) update current anchor
                {
                    let current_anchor = &mut anchors[current_anchor_index];
                    // update state
                    match block_type {
                        BlockType::Hind => {
                            current_anchor.state = AlignmentState::Exact(
                                None,
                                ExactAlign::Own(alignment_res),
                            );
                        },
                        BlockType::Fore => {
                            if let AlignmentState::Exact(fore_block, _) = &mut current_anchor.state {
                                *fore_block = Some(ExactAlign::Own(alignment_res));
                            }
                        }
                    }
                    // update connected anchors
                    current_anchor.connected.extend(connected_anchor_indices.iter());
                }
                // (5) update connected anchors
                for (connected_anchor_index, (length, penalty_in_ref)) in connected_backtraces {
                    let connected_anchor = &mut anchors[connected_anchor_index];
                    match block_type {
                        BlockType::Hind => {
                            // update anchor state
                            connected_anchor.state = AlignmentState::Exact(
                                None,
                                ExactAlign::Ref(
                                    current_anchor_index,
                                    (length, current_anchor_score - penalty_in_ref)
                                ),
                            );
                            // update anchor's connected info
                            for check_point in &connected_anchor.check_points.1 {
                                if connected_anchor_indices.contains(check_point) {
                                    connected_anchor.connected.insert(*check_point);
                                }
                            }
                        },
                        BlockType::Fore => {
                            // update anchor state
                            if let AlignmentState::Exact(fore_block, _) = &mut connected_anchor.state {
                                *fore_block = Some(ExactAlign::Ref(
                                    current_anchor_index,
                                    (length, current_anchor_score - penalty_in_ref)
                                ));
                            };
                            // update anchor's connected info
                            for check_point in &connected_anchor.check_points.0 {
                                if connected_anchor_indices.contains(check_point) {
                                    connected_anchor.connected.insert(*check_point);
                                }
                            };
                        }
                    }
                    // if anchor has cached wf: drop it
                    connected_anchor.wf_cache = None;
                }
            },
            /*
            CASE 2: wf dropped
            */
            // TODO:
            Err(wf) => {
                /* TODO: inherit
                if using_cached_wf {
                    let check_points_values = Self::wf_inheritance_check_points(anchors, current_anchor_index, ref_seq, qry_seq, block_type.clone());
                    // unpack map & sort by anchor index
                    let inheritable_checkpoints: Vec<(usize, usize, i32, i32, i32)> = {
                        let mut valid_checkpoints: Vec<(usize, usize, i32, i32, i32)> = wf_check_inheritable(&wf, penalties, check_points_values).into_iter().map(
                            |(key, val)| {
                                (key, val.0, val.1, val.2, val.3)
                            }
                        ).collect();
                        valid_checkpoints.sort_by(|a, b| a.cmp(&b));
                        valid_checkpoints
                    };
                    let mut checked_anchors_index: HashSet<usize> = HashSet::new();
                    for (anchor_index, score, k, fr, ext_fr) in inheritable_checkpoints {
                        // if anchor is not checked yet: caching WF
                        if !checked_anchors_index.contains(&anchor_index) {
                            let anchor = &mut anchors[anchor_index];
                            // inherit WF
                            anchor.wf_cache = Some(wf_inherited_cache(&wf, score, k, fr, ext_fr));
                            // add all check points to the checked index list
                            checked_anchors_index.insert(anchor_index);
                            match block_type {
                                BlockType::Hind => {
                                    checked_anchors_index.extend(anchor.check_points.1.iter());
                                },
                                BlockType::Fore => {
                                    checked_anchors_index.extend(anchor.check_points.0.iter());
                                },
                            }
                        }
                    }
                }
                */
                // drop current index
                anchors[current_anchor_index].to_dropped();
            },
        }
    }
    #[inline]
    fn to_dropped(&mut self) {
        self.state = AlignmentState::Dropped;
    }
    /**
    EVALUATE
    */
    #[inline]
    fn get_penalty_and_length(&self) -> (Penalty, SequenceLength) {
        let mut total_length: usize = 0;
        let mut total_penalty: usize = 0;
        if let AlignmentState::Exact(fore_option, hind) = &self.state {
            let fore = fore_option.as_ref().unwrap();
            // add fore & hind info
            for &exact_align in [fore, hind].iter() {
                let (len, p) = exact_align.length_and_penalty();
                total_length += len;
                total_penalty += p;
            }
        }
        total_length += self.size;
        (total_penalty, total_length)
    }
    #[inline]
    fn exact_alignment_is_valid(penalty: Penalty, length: SequenceLength, cutoff: &Cutoff) -> bool {
        if (length >= cutoff.minimum_length) && (penalty as f64/length as f64 <= cutoff.score_per_length) {
            true
        } else {
            false
        }
    }
    #[inline]
    fn get_unique_symbols(anchors: &Vec<Self>, anchors_of_minimum_penalty: Option<HashSet<usize>>) -> HashSet<usize> {
        // TODO: can be more optimized
        // valid anchors set
        let valid_anchors_set: HashSet<usize> = match anchors_of_minimum_penalty {
            Some(anchors_set) => anchors_set,
            None => {
                anchors.iter().enumerate().filter_map(
                    |(idx, anchor)| {
                        match anchor.state {
                            AlignmentState::Exact(_, _) => {
                                Some(idx)
                            },
                            _ => {
                                None
                            }
                        }
                    }
                ).collect()
            }
        };
        // symbol dictionary
        let anchor_symbols = {
            let mut anchor_symbols: HashMap<usize, HashSet<usize>> = HashMap::with_capacity(valid_anchors_set.len());
            // 1. add connected & valid anchor
            for &anchor_index in valid_anchors_set.iter() {
                let symbol: HashSet<usize> =  valid_anchors_set.intersection(&anchors[anchor_index].connected).map(|x| *x).collect();
                anchor_symbols.insert(anchor_index, symbol);
            };
            // 2. add extended anchors of connected
            for anchor_index in valid_anchors_set.iter() {
                let mut extended_symbol: HashSet<usize> = HashSet::new();
                anchor_symbols.get(anchor_index).unwrap().iter().for_each(|idx| {
                    extended_symbol.extend(anchor_symbols.get(idx).unwrap());
                });
                let symbol = anchor_symbols.get_mut(anchor_index).unwrap();
                symbol.extend(extended_symbol);
                // add self index
                symbol.insert(*anchor_index);
            };
            anchor_symbols
        };
        // unique symbols list
        let unique_anchor = {
            let mut unique_anchor: HashSet<usize> = HashSet::new();
            let mut used_symbols: HashSet<Vec<usize>> = HashSet::with_capacity(anchor_symbols.len());
            for (anchor_index, symbol) in anchor_symbols.into_iter() {
                let mut serialized_symbol: Vec<usize> = symbol.into_iter().collect();
                serialized_symbol.sort();
                if !used_symbols.contains(&serialized_symbol) {
                    unique_anchor.insert(anchor_index);
                    used_symbols.insert(serialized_symbol);
                }
            };
            unique_anchor
        };
        unique_anchor
    }
    #[inline]
    fn get_alignment<'a>(
        anchors: &'a Vec<Self>, current_anchor_index: usize,
        ref_len: SequenceLength, qry_len: SequenceLength
    ) -> Alignment {
        let current_anchor = &anchors[current_anchor_index];
        let mut total_length: usize = 0;
        let mut total_penalty: usize = 0;
        if let AlignmentState::Exact(fore_option, hind) = &current_anchor.state {
            // Closure: get cigar and reverse index
            let mut get_cigar_ridx = |exact_align: &'a ExactAlign| -> Option<(&'a Cigar, ReverseIndex)> {
                match exact_align {
                    ExactAlign::Own((cigar, length, penalty)) => {
                        if cigar.len() == 0 {
                            None
                        } else {
                            total_length += length;
                            total_penalty += penalty;
                            let mut reverse_index = get_reverse_index_from_own(cigar);
                            // update indel count to qry, ref left length
                            reverse_index.2 = *length as u32 - reverse_index.2;
                            reverse_index.3 = *length as u32 - reverse_index.3;
                            Some((cigar, reverse_index))
                        }
                    },
                    ExactAlign::Ref(ref_anchor_index, (length, penalty)) => {
                        total_length += length;
                        total_penalty += penalty;
                        if let AlignmentState::Exact(Some(ExactAlign::Own((cigar, _, _))), _) = &anchors[*ref_anchor_index].state {
                            let mut reverse_index = get_reverse_index_from_ref(cigar, length);
                            // update indel count to qry, ref left length
                            reverse_index.2 = *length as u32 - reverse_index.2;
                            reverse_index.3 = *length as u32 - reverse_index.3;
                            Some((cigar, get_reverse_index_from_ref(cigar, length)))
                        } else {
                            // TODO: err msg
                            panic!("Trying to get result operations from invalid anchor.");
                        }
                    }
                }
            };
            // Closure: Get clip operation
            let get_clip_op = |qry_aligned_length: u32, ref_aligned_length: u32| {
                let ref_left = ref_len - ref_aligned_length as SequenceLength;
                let qry_left = qry_len - qry_aligned_length as SequenceLength;
                if ref_left == qry_left {
                    Clip::None
                } else if ref_left > qry_left {
                    Clip::Ref(ref_left - qry_left)
                } else {
                    Clip::Qry(qry_left - ref_left)
                }
            };
            // (1) Get cigar & ridx
            let fore_cigar_ridx = get_cigar_ridx(fore_option.as_ref().unwrap());
            let hind_cigar_ridx = get_cigar_ridx(hind);
            // (2) Generate empty new cigar
            let mut new_cigar: Cigar = Vec::with_capacity(
                if let Some((_, (cigar_length, _, _, _))) = fore_cigar_ridx{
                    cigar_length
                } else {
                    0
                } +
                if let Some((_, (cigar_length, _, _, _))) = hind_cigar_ridx{
                    cigar_length
                } else {
                    0
                } + 1
            );
            // (3) Deal & generate clip with fore
            let fore_clip = match fore_cigar_ridx {
                Some((cigar, ridx)) => {
                    new_cigar.extend(&cigar[..ridx.0]);
                    new_cigar.last_mut().unwrap().1 = ridx.1;
                    get_clip_op(ridx.2, ridx.3)
                },
                None => {
                    Clip::None
                },
            };
            // (4) Deal with size
            if let Some((Operation::Match, count)) = new_cigar.last_mut() {
                *count += current_anchor.size as u32;
            } else {
                new_cigar.push((Operation::Match, current_anchor.size as u32))
            };
            total_length += current_anchor.size;
            // (5) Deal & generate clip with hind
            let hind_clip = match hind_cigar_ridx {
                Some((cigar, ridx)) => {
                    // first index
                    let first_cigar_op = cigar[ridx.0-1].0;
                    match first_cigar_op {
                        Operation::Match => {
                            new_cigar.last_mut().unwrap().1 += ridx.1;
                        },
                        other_op => {
                            new_cigar.push((other_op, ridx.1));
                        }
                    };
                    // extend remains
                    new_cigar.extend(cigar[..ridx.0-1].iter().rev());
                    get_clip_op(ridx.2, ridx.3)
                },
                None => {
                    Clip::None
                },
            };
            Alignment {
                penalty: total_penalty,
                length: total_length,
                clip_front: fore_clip,
                clip_end: hind_clip,
                cigar: new_cigar,
            }
        } else {
            // TODO: err msg
            panic!("Trying to get result operations from invalid anchor.");
        }
    }
}

#[derive(Clone)]
enum BlockType {
    Hind,
    Fore,
}
