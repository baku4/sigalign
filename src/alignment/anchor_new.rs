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

type IsHindBlock = bool;
const HIND_BLOCK: bool = true;
const FORE_BLOCK: bool = false;

// Anchors preset by one fasta record
#[derive(Debug, Clone)]
struct AnchorsPreset {
    pushed_positions: Vec<AnchorPosition>,
    pre_positions: Vec<AnchorPosition>,
}
#[derive(Debug, Clone)]
struct AnchorPosition {
    ref_pos: usize,
    qry_idx: usize,
    size: usize,
    used: bool,
}

impl AnchorsPreset {
    #[inline]
    fn new() -> Self {
        Self {
            pushed_positions: Vec::new(),
            pre_positions: Vec::new(),
        }
    }
    #[inline]
    fn estimate_penalty(&self, kmer: usize, forward_bp: &Vec<usize>, backward_bp: &Vec<usize>) {
        
    }
}

use crate::reference::Reference;

// Anchor Group
pub struct AnchorsGroup<'a> {
    anchors_by_seq: Vec<Vec<Anchor>>,
    reference: &'a Reference,
}
impl<'a> AnchorsGroup<'a> {
    fn new(
        penalties: &'a Penalties,
        cutoff: &'a Cutoff,
        block_penalty: &'a BlockPenalty,
        reference: &'a Reference,
        kmer: usize,
        query: &'a [u8]
    ) {
        let records_count: usize = reference.accumulated_length.len(); // FIXME: update this line
        let mut preset_container: Vec<AnchorsPreset> = vec![AnchorsPreset::new(); records_count];

        let qry_len = query.len();
        let search_count = qry_len / kmer;
        let mut anchor_existence: Vec<bool> = Vec::with_capacity(search_count); // first value is buffer
        // Create Preset of Anchors
        for qry_idx in 0..search_count {
            let qry_pos = qry_idx*kmer;
            let pattern = &query[qry_pos..qry_pos+kmer];
            let mut positions = reference.locate(pattern);
            if positions.is_empty() {
                anchor_existence.push(false);
                continue;
            } else {
                anchor_existence.push(true);
                positions.sort(); // must be sorted
            }
            let mut pos_idx: usize = 0;
            for j in 0..records_count {
                // Get AnchorsPreset
                let anchors_preset = &mut preset_container[j];
                // Make positions to add
                let mut curr_positions: Vec<AnchorPosition> = Vec::new();
                let mut ie_pre_positions: Vec<AnchorPosition> = Vec::new();
                // Fetch referece information
                let ref_last_index = &reference.accumulated_length[j];
                // Check if position can be added to this anchors preset
                while pos_idx < positions.len() {
                    let position = positions[pos_idx];
                    if position <= *ref_last_index {
                        if (position + kmer as u64) <= *ref_last_index { // the position is included in this record
                            let mut this_pos_is_ie = false;
                            // Check Impeccable Extension (IE)
                            for pre_pos in &mut anchors_preset.pre_positions {
                                if position as usize == (pre_pos.ref_pos + pre_pos.size) {
                                    let mut ie_pre_pos = pre_pos.clone();
                                    ie_pre_pos.size += kmer;
                                    ie_pre_positions.push(ie_pre_pos);
                                    this_pos_is_ie = true;
                                    pre_pos.used = true; // this pre position is used
                                    break;
                                }
                            }
                            // If not IE -> add to current positions cache
                            if !this_pos_is_ie {
                                curr_positions.push(
                                    AnchorPosition {
                                        ref_pos: position as usize,
                                        qry_idx: qry_idx,
                                        size: kmer,
                                        used: false,
                                    }
                                );
                            }
                            pos_idx += 1; // check next position with this anchors preset
                        } else { // this position traverse the edge of sequences
                            pos_idx += 1;
                            break; // escape the loop
                        }
                    } else { // the position is not included in this record
                        break; // pass this position to next record - escape the loop
                    }
                }
                // Add positions to this anchors preset
                // push unused pre positions
                for pre_pos in &anchors_preset.pre_positions {
                    if !pre_pos.used {
                        anchors_preset.pushed_positions.push(pre_pos.clone());
                    }
                }
                // ie & curr positions to pre positions
                ie_pre_positions.extend(curr_positions);
                anchors_preset.pre_positions = ie_pre_positions;
            }
        }
        // Push last pre_pos of anchors preset
        preset_container.iter_mut().for_each(|anchors_preset| {
            anchors_preset.pushed_positions.extend_from_slice(&anchors_preset.pre_positions);
            println!("pre pos is consumed?\n{:?}", anchors_preset.pre_positions); // TODO: to del
        });
        // Convert existence info to block penalty
        let (forward_bp, backward_bp) = existence_to_bp(anchor_existence, block_penalty);
        // AnchorsPreset to new anchors
        let mut anchors_by_seq = { // : Vec<Vec<Anchor>>
            preset_container.into_iter().for_each(|anchors_preset| {
                anchors_preset.estimate_penalty(kmer, &forward_bp, &backward_bp,);
            });
        };
        // print for testing
        // println!("{:#?}", anchor_existence); 
    }
}
struct AnchorNew {
    position: (SequenceLength, SequenceLength), // ref, qry
    size: SequenceLength,
    alignments: (AlignmentBlock, AlignmentBlock), // fore, hind
    check_points: (Vec<usize>, Vec<usize>), // fore, hind
    connected: HashSet<usize>,
}

enum AlignmentBlock {
    Estimated,
    Extended,
}

// Utils
fn existence_to_bp(anchor_existence: Vec<bool>, block_penalty: &BlockPenalty) -> (Vec<Penalty>, Vec<Penalty>) {
    let p_odd = block_penalty.odd;
    let p_even = block_penalty.even;
    // TODO: refine redundant code
    // forward_bp
    let mut last_is_odd = false;
    let forward_bp: Vec<Penalty> = anchor_existence.iter().map(|&exist| {
        if exist {
            if last_is_odd {
                p_even
            } else {
                p_odd
            }
        } else {
            last_is_odd = false;
            0
        }
    }).collect();
    // backward_bp
    let mut last_is_odd = false;
    let backward_bp: Vec<Penalty> = anchor_existence.iter().rev().map(|&exist| {
        if exist {
            if last_is_odd {
                p_even
            } else {
                p_odd
            }
        } else {
            last_is_odd = false;
            0
        }
    }).collect();
    (forward_bp, backward_bp)
}

// Anchor Group
#[derive(Debug)]
pub struct AnchorGroupDep<'a> {
    ref_seq: &'a [u8],
    qry_seq: &'a [u8],
    penalties: &'a Penalties,
    cutoff: &'a Cutoff,
    pub anchors: Vec<Anchor>,
}
impl<'a> AnchorGroupDep<'a> {
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
                let positions = index.locate_w_klt(pattern);
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
                HIND_BLOCK,
                using_cached_wf
            );
        }
        // (2) alignment fore
        for idx in (0..self.anchors.len()).rev() {
            Anchor::alignment(
                &mut self.anchors, idx,
                self.ref_seq, self.qry_seq, self.penalties, self.cutoff,
                FORE_BLOCK,
                using_cached_wf
            );
        };
    }
    pub fn get_result(&mut self, get_minimum_penalty: bool) -> AlignmentResult {
        // anchor index, length, penalty
        type AlignmentPreset = HashMap<usize, (SequenceLength, Penalty)>;
        // (1) Get AlignmentPreset of valid anchors
        let alignment_preset: AlignmentPreset = if get_minimum_penalty {
            let mut minimum_penalty: Penalty = Penalty::MAX;
            let mut anchors_map: AlignmentPreset = HashMap::new();
            for (anchor_index, anchor) in self.anchors.iter_mut().enumerate() {
                if let Some((length, penalty)) = anchor.drop_with_length_penalty(self.cutoff) {
                    if penalty < minimum_penalty {
                        minimum_penalty = penalty;
                        anchors_map = HashMap::new();
                        anchors_map.insert(anchor_index, (length, penalty));
                    } else if penalty == minimum_penalty {
                        anchors_map.insert(anchor_index, (length, penalty));
                    }
                }
            }
            anchors_map
        } else {
            let mut anchors_map: AlignmentPreset = HashMap::new();
            for (anchor_index, anchor) in self.anchors.iter_mut().enumerate() {
                if let Some((length, penalty)) = anchor.drop_with_length_penalty(self.cutoff) {
                    anchors_map.insert(anchor_index, (length, penalty));
                }
            }
            anchors_map
        };
        let valid_anchors: HashSet<usize> = alignment_preset.keys().map(|index| *index).collect();
        // (2) get unique anchors
        let unique_anchors = unique_symbols_filtering(valid_anchors, &self.anchors);
        // (3) get cigar & penalty
        let ref_len = self.ref_seq.len();
        let qry_len = self.qry_seq.len();
        unique_anchors.into_iter().map(|unique_anchor_index| {
            let (clip_front, clip_end, cigar) = Anchor::clips_and_cigar(
                &self.anchors,
                unique_anchor_index,
                ref_len,
                qry_len
            );
            let (length, penalty) = alignment_preset.get(&unique_anchor_index).unwrap();
            Alignment {
                penalty: *penalty,
                length: *length,
                clip_front: clip_front, 
                clip_end: clip_end,
                cigar: cigar, 
            }
        }).collect()
    }
}

/// Anchor
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
enum AlignmentState {
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
#[derive(Debug, Clone)]
struct EstAlign {
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
#[derive(Debug, Clone)]
enum ExactAlign {
    /// Having an operations.
    Own((Cigar, SequenceLength, Penalty)), 
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
    #[inline]
    fn get_cigar_ridx<'a>(
        &'a self,
        anchors: &'a Vec<Anchor>,
        is_hind_block: IsHindBlock,
        ref_len: usize,
        qry_len: usize,
    ) -> (Option<(&'a Cigar, usize, u32)>, Clip) { // (cigar, cigar length, offset), Clip
        match self {
            ExactAlign::Own((cigar, length, _)) => {
                if cigar.len() == 0 {
                    (None, Clip::new(
                        ref_len,
                        qry_len,
                        *length,
                        *length,
                    ))
                } else {
                    let (cigar_length, offset, ins, del) = get_reverse_index_from_own(cigar);
                    (Some((cigar, cigar_length, offset)), Clip::new(
                        ref_len,
                        qry_len,
                        *length - del as usize,
                        *length - ins as usize,
                    ))
                }
            },
            ExactAlign::Ref(ref_anchor_index, (length, _)) => {
                let cigar = if is_hind_block {
                    if let AlignmentState::Exact(_,ExactAlign::Own((cigar, _, _))) = &anchors[*ref_anchor_index].state {
                        cigar
                    } else {
                        // TODO: err msg
                        panic!("Trying to get result operations from invalid anchor.");
                    }
                } else {
                    if let AlignmentState::Exact(Some(ExactAlign::Own((cigar, _, _))), _) = &anchors[*ref_anchor_index].state {
                        cigar
                    } else {
                        // TODO: err msg
                        panic!("Trying to get result operations from invalid anchor.");
                    }
                };
                let (cigar_length, offset, ins, del) = get_reverse_index_from_ref(cigar, length);
                (Some((cigar, cigar_length, offset)), Clip::new(
                    ref_len,
                    qry_len,
                    *length - del as usize,
                    *length - ins as usize,
                ))
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
            if length >= cutoff.ml && (emp_block_1.penalty + emp_block_2.penalty) as f64/length as f64 <= cutoff.ppl {
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
            if (penalty as f64 / length as f64 <= cutoff.ppl) & (length >= cutoff.ml) {
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
    fn get_check_points_for_backtrace(anchors: &Vec<Self>, current_index: usize, is_hind_block: IsHindBlock) -> AnchorsToPassCheck {
        let current_anchor = &anchors[current_index];
        if is_hind_block {
            let check_points = &current_anchor.check_points.1;
            let mut backtrace_check_points: AnchorsToPassCheck = Vec::with_capacity(check_points.len());
            check_points.into_iter().for_each(|&anchor_index| {
                let anchor = &anchors[anchor_index];
                let ref_gap = (anchor.position.0 + anchor.size - current_anchor.position.0 - current_anchor.size) as i32;
                let qry_gap = (anchor.position.1 + anchor.size - current_anchor.position.1 - current_anchor.size) as i32;
                backtrace_check_points.push((anchor_index, anchor.size as i32, ref_gap-qry_gap, ref_gap));
            });
            backtrace_check_points
        } else {
            let check_points = &current_anchor.check_points.0;
            let mut backtrace_check_points: AnchorsToPassCheck = Vec::with_capacity(check_points.len());
            check_points.into_iter().for_each(|&anchor_index| {
                let anchor = &anchors[anchor_index];
                let ref_gap = (current_anchor.position.0 - anchor.position.0) as i32;
                let qry_gap = (current_anchor.position.1 - anchor.position.1) as i32;
                backtrace_check_points.push((anchor_index, anchor.size as i32, ref_gap-qry_gap, ref_gap));
            });
            backtrace_check_points
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
        is_hind_block: IsHindBlock, using_cached_wf: bool,
    ) {
        // TODO: to del
        // #[cfg(test)]
        // {
        //     println!("current index: {:?} / pos: {:?}", current_anchor_index, anchors[current_anchor_index].position);
        // }
        // (1) DWFA
        let dwfa_res = {
            // get refernce of current anchor
            let current_anchor = &mut anchors[current_anchor_index];
            let (l_opp, p_opp) = if is_hind_block {
                if let AlignmentState::Estimated(est_align, _) = &current_anchor.state {
                    (est_align.length, est_align.penalty)
                } else {
                    return;
                }
            } else {
                if let AlignmentState::Exact(None, hind) = &current_anchor.state {
                    hind.length_and_penalty()
                } else {
                    return;
                }
            };
            let penalty_spare = if is_hind_block {
                (
                    penalties.e as f64 * cutoff.ppl * (
                        l_opp + current_anchor.size + min(
                            ref_seq.len() - current_anchor.position.0 - current_anchor.size,
                            qry_seq.len() - current_anchor.position.1 - current_anchor.size,
                        )
                    ) as f64
                    - (penalties.e * p_opp) as f64
                    - penalties.o as f64 * cutoff.ppl
                ) / (penalties.e as f64 - cutoff.ppl)
            } else {
                (
                    penalties.e as f64 * cutoff.ppl * (
                        l_opp + current_anchor.size + min(
                            current_anchor.position.0,
                            current_anchor.position.1,
                        )
                    ) as f64
                    - ( penalties.e * p_opp) as f64
                    - penalties.o as f64 * cutoff.ppl
                ) / (penalties.e as f64 - cutoff.ppl)
            }.ceil() as usize;
            // Get cached wf
            // if current anchor has cached wf -> continue with cached wf
            let wf_cache = current_anchor.wf_cache.take();
            // TODO: to del
            // #[cfg(test)]
            // {
            //     if let Some(v) = &wf_cache {
            //         println!("using inherited wf:\n{:?}\n{:?}, ", v[0], v[1]);
            //     };
            // }
            // DWFA
            if is_hind_block {
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
                            penalties,
                            true,
                        )
                    },
                }
            } else {
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
                            &qry_seq[..current_anchor.position.1],
                            &ref_seq[..current_anchor.position.0],
                            penalty_spare,
                            penalties,
                            false,
                        )
                    },
                }
            }
        };
        // (2) BACKTRACE
        match dwfa_res {
            /*
            CASE 1: wf not dropped
            */
            Ok((mut wf, score, last_k)) => {
                // (1) get check points for backtrace
                let anchors_to_chk: AnchorsToPassCheck = Self::get_check_points_for_backtrace(
                    anchors, current_anchor_index, is_hind_block.clone()
                );
                // (2) bactrace
                let (alignment_res, connected_backtraces) = dropout_wf_backtrace(
                    &wf, penalties, score, last_k, &anchors_to_chk
                );
                // (3) get valid anchor index
                let connected_anchor_indices: HashSet<usize> = HashSet::from_iter(
                    connected_backtraces.keys().map(|x| *x)
                );
                // (4) update current anchor
                {
                    let current_anchor = &mut anchors[current_anchor_index];
                    // update state
                    if is_hind_block {
                        current_anchor.state = AlignmentState::Exact(
                            None,
                            ExactAlign::Own((alignment_res.0, alignment_res.1, score)),
                        );
                    } else {
                        if let AlignmentState::Exact(fore_block, _) = &mut current_anchor.state {
                            *fore_block = Some(ExactAlign::Own((alignment_res.0, alignment_res.1, score)));
                        }
                    }
                    // update connected anchors
                    current_anchor.connected.extend(connected_anchor_indices.iter());
                }
                // (5) update connected anchors
                for (connected_anchor_index, (length, penalty_in_ref)) in connected_backtraces {
                    let connected_anchor = &mut anchors[connected_anchor_index];
                    if is_hind_block {
                        // update anchor state
                        connected_anchor.state = AlignmentState::Exact(
                            None,
                            ExactAlign::Ref(
                                current_anchor_index,
                                (length, score - penalty_in_ref)
                            ),
                        );
                        // update anchor's connected info
                        for check_point in &connected_anchor.check_points.1 {
                            if connected_anchor_indices.contains(check_point) {
                                connected_anchor.connected.insert(*check_point);
                            }
                        }
                    } else {
                        // update anchor state
                        if let AlignmentState::Exact(fore_block, _) = &mut connected_anchor.state {
                            *fore_block = Some(ExactAlign::Ref(
                                current_anchor_index,
                                (length, score - penalty_in_ref)
                            ));
                        };
                        // update anchor's connected info
                        for check_point in &connected_anchor.check_points.0 {
                            if connected_anchor_indices.contains(check_point) {
                                connected_anchor.connected.insert(*check_point);
                            }
                        };
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
    /*
    EVALUATE
    */
    #[inline]
    fn drop_with_length_penalty(&mut self, cutoff: &Cutoff) -> Option<(SequenceLength, Penalty)> {
        let mut total_length: usize = 0;
        let mut total_penalty: usize = 0;
        // If already dropped: pass
        if let AlignmentState::Exact(fore_option, hind) = &self.state {
            // fore
            let (len, p) = fore_option.as_ref().unwrap().length_and_penalty();
            total_length += len;
            total_penalty += p;
            // anchor
            total_length += self.size;
            // hind
            let (len, p) = hind.length_and_penalty();
            total_length += len;
            total_penalty += p;
            
        } else {
            return None;
        }
        // Evaluate
        if (total_length >= cutoff.ml) && (total_penalty as f64/total_length as f64 <= cutoff.ppl) {
            Some((total_length, total_penalty))
        } else {
            self.to_dropped();
            None
        }
    }
    #[inline]
    fn clips_and_cigar<'a>(
        anchors: &'a Vec<Self>, current_anchor_index: usize,
        ref_len: SequenceLength, qry_len: SequenceLength
    ) -> (Clip, Clip, Cigar) {
        let current_anchor = &anchors[current_anchor_index];
        if let AlignmentState::Exact(fore_option, hind) = &current_anchor.state {
            // (1) Get cigar & ridx
            let (fore_cigar, fore_clip) = fore_option.as_ref().unwrap().get_cigar_ridx(
                anchors,
                FORE_BLOCK,
                current_anchor.position.0,
                current_anchor.position.1
            );
            let (hind_cigar, hind_clip) = hind.get_cigar_ridx(
                anchors,
                HIND_BLOCK,
                ref_len-current_anchor.position.0-current_anchor.size,
                qry_len-current_anchor.position.1-current_anchor.size
            );
            // (2) Generate empty new cigar
            let mut new_cigar: Cigar = Vec::with_capacity(
                if let Some((_, cigar_length, _)) = fore_cigar{
                    cigar_length
                } else {
                    0
                } +
                if let Some((_, cigar_length, _)) = hind_cigar{
                    cigar_length
                } else {
                    0
                } + 1
            );
            // (3) Deal & generate clip with fore
            if let Some((cigar, cigar_length, offset)) = fore_cigar {
                new_cigar.extend(&cigar[..cigar_length]);
                new_cigar.last_mut().unwrap().1 = offset;
            };
            // (4) Deal with size
            if let Some((Operation::Match, count)) = new_cigar.last_mut() {
                *count += current_anchor.size as u32;
            } else {
                new_cigar.push((Operation::Match, current_anchor.size as u32))
            };
            // (5) Deal & generate clip with hind
            if let Some((cigar, cigar_length, offset)) = hind_cigar{
                // first index
                let first_cigar_op = cigar[cigar_length-1].0;
                match first_cigar_op {
                    Operation::Match => {
                        new_cigar.last_mut().unwrap().1 += offset;
                    },
                    other_op => {
                        new_cigar.push((other_op, offset));
                    }
                };
                // extend remains
                new_cigar.extend(cigar[..cigar_length-1].iter().rev());
            };
            (fore_clip, hind_clip, new_cigar)
        } else {
            // TODO: err msg
            panic!("Trying to get result operations from invalid anchor.");
        }
    }
}

#[inline]
fn unique_symbols_filtering(
    valid_anchors: HashSet<usize>,
    anchors: &Vec<Anchor>,
) -> HashSet<usize>{
    // init
    let mut anchor_symbols: HashMap<usize, HashSet<usize>> = HashMap::with_capacity(valid_anchors.len());
    let mut used_symbols: HashSet<Vec<usize>> = HashSet::with_capacity(valid_anchors.len());
    let mut unique_anchors: HashSet<usize> = HashSet::with_capacity(valid_anchors.len());
    // 1. add connected & valid anchor
    for &anchor_index in valid_anchors.iter() {
        let symbol: HashSet<usize> = valid_anchors.intersection(&anchors[anchor_index].connected).map(|x| *x).collect();
        anchor_symbols.insert(anchor_index, symbol);
    };
    // 2. get unique anchors
    for (anchor_index, mut symbol) in anchor_symbols.clone() {
        // get lexicographic ordered symbols
        let lexico_ordered_symbol = {
            // add extended anchors 
            for extended_anchor_index in symbol.clone() {
                for index in anchor_symbols.get(&extended_anchor_index).unwrap() {
                    symbol.insert(*index);
                }
            }
            // add self
            symbol.insert(anchor_index);
            // symbol to vector
            let mut vec: Vec<usize> = symbol.into_iter().collect();
            vec.sort();
            vec
        };
        // if not exist in used_symbols: add to unique anchors set
        if !used_symbols.contains(&lexico_ordered_symbol) {
            used_symbols.insert(lexico_ordered_symbol);
            unique_anchors.insert(anchor_index);
        }
    }
    unique_anchors
}


#[cfg(test)]
mod tests {
    use lt_fm_index::{FmIndexConfig};

    use std::mem::align_of;

    use crate::*;
    use crate::reference::*;
    use crate::alignment::*;
    use super::*;

    fn text_1000_on() -> Vec<u8> {
        "CTCCGTGATTTACCGGATGCATAGATTTCCCCATTTTGCGTACCGGAAACGTGCGCAAGCACGATCTGTGTCTTACCCTCCGTACACCTGTTTCGTATCGGAACCGGTAAACACCTGTTTCGTATCGGAACCGGTAAGTGAAATTTCCACATCGCCGGAAACCGTATATTGTCCATCCGCTGCCGGTTCCGTACACCTGTTTCGTATCGGAACCGGTAAGTGAAATTTCCACATCGCCGGAAACCGTATATTGTCCATCCGCTGCCGGTGGATCCGGCTCCTGCGTGGAAAACCAGTCATCCTGATTTACATATGGTTCAATGGCACCGGATTGTTGCTGGTTCAATGGCACCGGATGCATAGATTTCCCCATTTTGCGTACCGGAAACGTGCGCAAGCACGATCTGTGTCTTACCCTCCGTACACCTGTTTCGATTTACCGGATGCATAGATTTCCCCATTTTGCGTACCGGAAACGTGCGCAAGCACGATCTGTGTCTTACCCTCCGTACACCTGTTTCGTATCGGAACCGGTAAGTATCGGAACCGGTAAGTGAAATTTCCACATCGCCGGAAACCGTATATTGTCCATCCGCTGCCGGTGGATCCGGCTCCTGCGTGGAAAACCAGTCATCCTGATTTACATATGGTTTTCCACATCGCCGGAAACCGTATATTGTCCATCCGCTGCCGGTGGATCCGGCTCCTGCGTGGAAAACCAGTCATCCTGATTTACATATGGTTCAATGGCACCGGATGCATAGATTTCGTACCGGAAACGTGCGCAAGCACGATCTGTGTCTTACCCTCCGTACACCTGTTTCGTATCGGAACCGGTAAGTGATTTACCGGATGCATAGATTTCCCCATTTTGCGTACCGGAAACGTGCGCAAGCACGATCTGTGTCTTACCCTCCGTACACCTGTTTCGTATCGGAACCGGTAAAAATTTCCACATCGCCGGAAACCGTATATTGTCCATCCGCTGCCGGTGGATCCG".as_bytes().to_vec()
    }
    fn pattern_100_on() -> Vec<u8> {
       b"CGTGGAAAACCAGTCATCCTGATTTACATATGGTTCAATGGCACCGGATTGTTGCTGGTTCAATGGCACCGGATGCATAGATTTCCCCATTTTGCGTACC".to_vec()
    }

    #[test]
    fn test_new_anchors_preset() {
        let text = text_1000_on();

        let cutoff = Cutoff::new(50, 0.1);
        let penalties = Penalties::new(4,6,2);
        let reference = ReferenceConfig::new()
            .contain_only_nucleotide(true)
            .search_reverse_complement(true)
            .set_kmer_size_for_klt(8)
            .set_sampling_ratio_for_sa(2)
            .generate_reference_with_string(text.clone());
        
        let aligner = Aligner::new(cutoff, penalties);
        
        let query = b"TTACCACATCGCCGGAAACCGCATATTGTCCATCCGCTGCCGGTGGATCCGAAACCGTATATTGTCCATCC";


        //new
        let _ = AnchorsGroup::new(&aligner.penalties, &aligner.cutoff, &aligner.block_penalty, &reference, aligner.kmer, query);


        // old
        let fm_index = FmIndexConfig::new()
            .set_kmer_lookup_table(7)
            .generate_fmindex(text.clone());
        let anchors_group_old = AnchorGroupDep::new(&text, query, &fm_index, aligner.kmer, &aligner.block_penalty, &aligner.penalties, &aligner.cutoff);

        println!("{:#?}", anchors_group_old.unwrap().anchors);
    }
    // to be connected ->
    // (anchor last pos < next anchor first pos)
}