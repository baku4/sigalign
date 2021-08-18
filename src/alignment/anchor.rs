use crate::database::{
    Database, AccumulatedLength, SearchRange,
};
use crate::{SequenceLength, Penalty};
use super::dwfa::{
    WaveFront, AnchorsToPassCheck, CigarReference, BacktraceResult, RefToBacktrace,
    dropout_wf_align, dropout_wf_backtrace
};
use super::operation::{
    AlignedBlock, Operation, Clip, ReverseIndex,
    get_reverse_index_from_own, get_reverse_index_from_ref,
};
use super::{Penalties, Cutoff, BlockPenalty, AlignmentResult, AlignmentResultForDB};

use std::time::{Duration, Instant};
use core::panic;
use std::cmp::{min, max};
use std::collections::{HashMap, HashSet};
use std::iter::FromIterator;

/*****************************
******** ANCHOR GROUP ********
*****************************/
#[derive(Debug)]
struct AnchorsGroup {
    anchors_by_index: HashMap<usize, Vec<Anchor>>,
}
impl AnchorsGroup {
    #[inline]
    pub fn new(
        database: &Database,
        search_range: &SearchRange,
        cutoff: &Cutoff,
        block_penalty: &BlockPenalty,
        kmer: usize,
        query: &[u8],
    ) -> Self {
        // init
        let qry_len = query.len();
        let search_count = qry_len / kmer;
        let mut anchor_existence: Vec<bool> = Vec::with_capacity(search_count);

        let mut anchors_preset_by_index: HashMap<usize, AnchorsPreset> = HashMap::new();
        /*
        (1) Create Preset of Anchors
        */
        for qry_idx in 0..search_count {
            let qry_pos = qry_idx*kmer;
            // Get positions of reference
            let pattern = &query[qry_pos..qry_pos+kmer];
            let mut positions = database.locate(pattern);
            if positions.is_empty() {
                anchor_existence.push(false);
                continue;
            } else {
                anchor_existence.push(true);
                positions.sort(); // must be sorted
            }
            let ref_positions_by_index = database.find_ref_positions(
                search_range, positions, kmer as u64
            );

            // Ref pos to preset of anchors
            for (ref_index, ref_positions) in ref_positions_by_index {
                match anchors_preset_by_index.get_mut(&ref_index) {
                    Some(anchors_preset) => {
                        // anchors_preset
                        anchors_preset.add_anchors(ref_positions, qry_idx);
                    },
                    None => {
                        let ref_len = database.get_ref_len(ref_index);
                        let mut new_anchors_preset = AnchorsPreset::new(ref_len);
                        new_anchors_preset.add_anchors(ref_positions, qry_idx);
                        anchors_preset_by_index.insert(ref_index, new_anchors_preset);
                    },
                }
            }
        }
        /*
        (2) Preset to new anchors
        */
        let (forward_bp, backward_bp) = AnchorsPreset::existence_to_bp(anchor_existence, block_penalty);
        let anchors_by_index: HashMap<usize, Vec<Anchor>> = anchors_preset_by_index.into_iter().map(|(index, anchors_preset)| {
            (index, anchors_preset.to_anchors(kmer, qry_len, &forward_bp, &backward_bp, cutoff))
        }).collect();
        Self {
            anchors_by_index: anchors_by_index,
        }
    }
    #[inline]
    pub fn alignment(
        &mut self,
        database: &Database,
        penalties: &Penalties,
        cutoff: &Cutoff,
        query: &[u8],
        get_minimum_penalty: bool,
    ) -> AlignmentResultForDB {
        type AlignmentPreset = HashMap<usize, (usize, usize)>; // anchor index, (penalty, length)

        let mut result_for_db: AlignmentResultForDB = HashMap::with_capacity(self.anchors_by_index.len());
        let mut minimum_penalty: Penalty = Penalty::MAX;

        for (index, anchors) in &mut self.anchors_by_index {
            let ref_seq = database.get_sequence(*index);
            // (1) Alignment hind
            for idx in 0..anchors.len() {
                Anchor::alignment(
                    anchors,
                    idx,
                    ref_seq,
                    query,
                    penalties,
                    cutoff,
                    HIND_BLOCK,
                );
            }
            // (2) Alignment fore
            for idx in (0..anchors.len()).rev() {
                Anchor::alignment(
                    anchors,
                    idx,
                    ref_seq,
                    query,
                    penalties,
                    cutoff,
                    FORE_BLOCK,
                );
            }
            // (3) Get AlignmentPreset of valid anchors
            let alignment_preset: AlignmentPreset = if get_minimum_penalty {
                let mut anchors_map: AlignmentPreset = HashMap::new();
                for (anchor_index, anchor) in anchors.iter_mut().enumerate() {
                    if let Some((penalty, length)) = anchor.drop_with_penalty_length(cutoff) {
                        if penalty < minimum_penalty {
                            minimum_penalty = penalty;
                            anchors_map.clear();
                            anchors_map.insert(anchor_index, (penalty, length));
                        } else if penalty == minimum_penalty {
                            anchors_map.insert(anchor_index, (penalty, length));
                        }
                    }
                }
                anchors_map
            } else {
                let mut anchors_map: AlignmentPreset = HashMap::new();
                for (anchor_index, anchor) in anchors.iter_mut().enumerate() {
                    if let Some((penalty, length)) = anchor.drop_with_penalty_length(cutoff) {
                        anchors_map.insert(anchor_index, (penalty, length));
                    }
                }
                anchors_map
            };
            let valid_anchors: HashSet<usize> = alignment_preset.keys().map(|index| *index).collect();
            // (4) Get unique anchors
            let unique_anchors = unique_symbols_filtering(valid_anchors, &*anchors);
            // (5) Get cigar & penalty
            let ref_len = ref_seq.len();
            let qry_len = query.len();
            let res: Vec<AlignmentResult> = unique_anchors.into_iter().map(|unique_anchor_index| {
                let (clip_front, clip_end, aligned_block) = Anchor::clips_and_cigar(
                    anchors,
                    unique_anchor_index,
                    ref_len,
                    qry_len
                );
                let (penalty, length) = alignment_preset.get(&unique_anchor_index).unwrap();
                AlignmentResult {
                    penalty: *penalty,
                    length: *length,
                    clip_front: clip_front, 
                    clip_end: clip_end,
                    aligned_block: aligned_block, 
                }
            }).collect();
            if res.len() != 0 {
                result_for_db.insert(*index, res);
            }
        }
        result_for_db
    }
}

/*****************************
******* ANCHOR PRESET ********
*****************************/
#[derive(Debug, Clone)]
struct AnchorsPreset {
    ref_len: usize,
    positions: Vec<(usize, Vec<usize>)>, // (qry idx, ref positions)
}
impl AnchorsPreset {
    #[inline]
    fn new(ref_len: usize) -> Self {
        Self { ref_len, positions: Vec::new() }
    }
    #[inline]
    fn add_anchors(&mut self, ref_positions: Vec<usize>, qry_idx: usize) {
        self.positions.push((qry_idx, ref_positions));
    }
    #[inline]
    fn to_anchors(
        self,
        kmer: usize,
        qry_len: usize,
        forward_bp: &Vec<usize>,
        backward_bp: &Vec<usize>,
        cutoff: &Cutoff,
    ) -> Vec<Anchor> {
        let mut anchors: Vec<Anchor> = Vec::new(); // TODO: with cap?
        let mut to_check_indices: Vec<usize> = Vec::new();
        let mut pre_idx = 0;
        let mut qry_pos;
        for (qry_idx, ref_positions) in self.positions {
            let anchors_pre_count = anchors.len();
            let mut added_anchors_count = 0;
            let mut used_pre_indices: Vec<usize> = Vec::new();
            qry_pos = qry_idx * kmer;
            if pre_idx + 1 == qry_idx {
                'ref_check: for ref_pos in ref_positions {
                    // if break here: IE
                    for to_check_idx in &to_check_indices {
                        let pre_anchor = &mut anchors[*to_check_idx];
                        if pre_anchor.position.0 + pre_anchor.size == ref_pos {
                            pre_anchor.size += kmer;
                            pre_anchor.hind.reduce_hind_size(kmer);
                            used_pre_indices.push(*to_check_idx);
                            continue 'ref_check;
                        }
                    }
                    // Not IE
                    let mut total_length = 0;
                    let mut total_penalty = 0;
                    // length & penalty of fore block
                    let fore_block = {
                        let mut length = min(ref_pos, qry_pos);
                        let block_count = length / kmer;
                        let penalty: usize = backward_bp[qry_idx-block_count..qry_idx].iter().map(|p| {
                            if *p != 0 { length += 1; };
                            *p
                        }).sum();
                        total_length += length;
                        total_penalty += penalty;
                        AlignmentBlock::Estimated(penalty, length)
                    };
                    // length & penalty of hind block
                    let hind_block = {
                        let ref_left = self.ref_len - ref_pos;
                        let qry_left = qry_len - qry_pos;
                        let mut length = min(ref_left, qry_left);
                        let block_count = length / kmer;
                        let penalty: usize = forward_bp[qry_idx..qry_idx+block_count].iter().map(|p| {
                            if *p != 0 { length += 1; };
                            *p
                        }).sum();
                        total_length += length;
                        total_penalty += penalty;
                        AlignmentBlock::Estimated(penalty, length-kmer)
                    };
                    if (total_length >= cutoff.ml) && (total_penalty as f64/total_length as f64) <= cutoff.ppl {
                        let new_anchor = Anchor {
                            position: (ref_pos, qry_pos), // ref, qry
                            size: kmer,
                            fore: fore_block,
                            hind: hind_block,
                            check_points: (Vec::new(), Vec::new()), // fore, hind
                            connected: HashSet::new(),
                            dropped: false,
                        };
                        added_anchors_count += 1;
                        anchors.push(new_anchor);
                    }
                }
            } else { // TODO: deredundant
                for ref_pos in ref_positions {
                    let mut total_length = 0;
                    let mut total_penalty = 0;
                    // length & penalty of fore block
                    let fore_block = {
                        let mut length = min(ref_pos, qry_pos);
                        let block_count = length / kmer;
                        let penalty: usize = backward_bp[qry_idx-block_count..qry_idx].iter().map(|p| {
                            if *p != 0 { length += 1; };
                            *p
                        }).sum();
                        total_length += length;
                        total_penalty += penalty;
                        AlignmentBlock::Estimated(penalty, length)
                    };
                    // length & penalty of hind block
                    let hind_block = {
                        let ref_left = self.ref_len - ref_pos;
                        let qry_left = qry_len - qry_pos;
                        let mut length = min(ref_left, qry_left);
                        let block_count = length / kmer;
                        let penalty: usize = forward_bp[qry_idx..qry_idx+block_count].iter().map(|p| {
                            if *p != 0 { length += 1; };
                            *p
                        }).sum();
                        total_length += length;
                        total_penalty += penalty;
                        AlignmentBlock::Estimated(penalty, length-kmer)
                    };
                    if (total_length >= cutoff.ml) && (total_penalty as f64/total_length as f64) <= cutoff.ppl {
                        let new_anchor = Anchor {
                            position: (ref_pos, qry_pos), // ref, qry
                            size: kmer,
                            fore: fore_block,
                            hind: hind_block,
                            check_points: (Vec::new(), Vec::new()), // fore, hind
                            connected: HashSet::new(),
                            dropped: false,
                        };
                        added_anchors_count += 1;
                        anchors.push(new_anchor);
                    }
                }
            }
            used_pre_indices.extend(anchors_pre_count..anchors_pre_count+added_anchors_count);
            to_check_indices = used_pre_indices;
            pre_idx = qry_idx;
        }
        anchors
    }

    #[inline]
    fn existence_to_bp(anchor_existence: Vec<bool>, block_penalty: &BlockPenalty) -> (Vec<Penalty>, Vec<Penalty>) {
        let p_odd = block_penalty.odd;
        let p_even = block_penalty.even;
        // TODO: refine redundant code
        // forward_bp
        let mut last_is_odd = false;
        let forward_bp: Vec<Penalty> = anchor_existence.iter().map(|&exist| {
            if exist {
                last_is_odd = false; 0
            } else {
                if last_is_odd { 
                    last_is_odd = false; p_even
                } else {
                    last_is_odd = true; p_odd
                }
            }
        }).collect();
        // backward_bp
        let mut last_is_odd = false;
        let backward_bp: Vec<Penalty> = anchor_existence.iter().rev().map(|&exist| {
            if exist {
                last_is_odd = false; 0
            } else {
                if last_is_odd { 
                    last_is_odd = false; p_even
                } else {
                    last_is_odd = true; p_odd
                }
            }
        }).rev().collect();
        (forward_bp, backward_bp)
    }
}

type IsHindBlock = bool;
const HIND_BLOCK: bool = true;
const FORE_BLOCK: bool = false;

/*****************************
********** ANCHOR ************
*****************************/

#[derive(Debug, Clone)]
struct Anchor { 
    position: (usize, usize), // ref, qry
    size: usize,
    fore: AlignmentBlock,
    hind: AlignmentBlock,
    check_points: (Vec<usize>, Vec<usize>), // fore, hind
    connected: HashSet<usize>,
    dropped: bool,
}

#[derive(Debug, Clone)]
enum AlignmentBlock {
    Estimated(usize, usize), // penalty & length
    Extended(ExactAlignment),
}
impl AlignmentBlock {
    #[inline]
    fn penalty_and_length(&self) -> (usize, usize) {
        match self {
            Self::Estimated(p, l) => (*p, *l),
            Self::Extended(ea) => {
                match ea {
                    ExactAlignment::Own(_, v) => *v,
                    ExactAlignment::Ref(_, v) => *v,
                }
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
    ) -> (Option<(&'a AlignedBlock, usize, u32)>, Clip) { // (cigar, cigar length, offset), Clip
        match self {
            Self::Estimated(_, _) => {
                panic!("Try to get rdix from estimated block") //TODO: err msg
            },
            Self::Extended(ea) => {
                ea.get_cigar_ridx(anchors, is_hind_block, ref_len, qry_len)
            }
        }
    }
    #[inline]
    fn reduce_hind_size(&mut self, kmer: usize) {
        if let Self::Estimated(_, l) = self {
            *l -= kmer;
        };
    }
}
#[derive(Debug, Clone)]
enum ExactAlignment {
    Own(AlignedBlock, (usize, usize)), // AlignedBlock, (penalty, length)
    Ref(usize, (usize, usize)), // index of connected anchor, (penalty, length)
}
impl ExactAlignment {
    #[inline]
    fn get_cigar_ridx<'a>(
        &'a self,
        anchors: &'a Vec<Anchor>,
        is_hind_block: IsHindBlock,
        ref_len: usize,
        qry_len: usize,
    ) -> (Option<(&'a AlignedBlock, usize, u32)>, Clip) { // (cigar, cigar length, offset), Clip
        match self {
            ExactAlignment::Own(cigar, (_, length)) => {
                if cigar.len() == 0 {
                    (None, Clip::new(
                        ref_len,
                        qry_len,
                        *length,
                        *length,
                    ))
                } else {
                    let (cigar_length, offset, ins, del) = get_reverse_index_from_own(cigar);
                    (
                        Some((cigar, cigar_length, offset)),
                        Clip::new(
                            ref_len,
                            qry_len,
                            *length - del as usize,
                            *length - ins as usize,
                        )
                    )
                }
            }
            ExactAlignment::Ref(ref_anchor_index, (_, length)) => {
                let cigar = if is_hind_block {
                    if let AlignmentBlock::Extended(ExactAlignment::Own(cigar, _)) = &anchors[*ref_anchor_index].hind {
                        cigar
                    } else {
                        // TODO: err msg
                        panic!("Trying to get result operations from invalid anchor.");
                    }
                } else {
                    if let AlignmentBlock::Extended(ExactAlignment::Own(cigar, _)) = &anchors[*ref_anchor_index].fore {
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
    CHECK POINTS
    */
    #[inline]
    fn create_check_points(anchors_record: &mut Vec<Self>, penalties: &Penalties, cutoff: &Cutoff) {
        let anchor_count = anchors_record.len();
        for fore_idx in 0..anchor_count {
            for hind_idx in fore_idx+1..anchor_count {
                if Self::can_be_connected(
                    &anchors_record[fore_idx],
                    &anchors_record[hind_idx],
                    penalties,
                    cutoff,
                ) {
                    anchors_record[fore_idx].check_points.1.push(hind_idx);
                    anchors_record[hind_idx].check_points.0.push(fore_idx);
                }
            }
        }
    }
    #[inline]
    fn can_be_connected(fore: &Self, hind: &Self, penalties: &Penalties, cutoff: &Cutoff) -> bool {
        let ref_gap = hind.position.0 as i64 - fore.position.0 as i64 - fore.size as i64;
        let qry_gap = hind.position.1 as i64 - fore.position.1 as i64 - fore.size as i64;
        if (ref_gap >= 0) && (qry_gap >= 0) {
            let mut penalty: usize = 0;
            let mut length: usize = 0;
            // fore
            if let AlignmentBlock::Estimated(p, l) = &fore.fore {
                penalty += p;
                length += l;
            }
            // hind
            if let AlignmentBlock::Estimated(p, l) = &hind.hind {
                penalty += p;
                length += l;
            }
            // middle
            length += max(ref_gap, qry_gap) as usize + fore.size + hind.size;
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
    /*
    ALIGNMENT
    */
    #[inline]
    fn alignment(
        anchors: &mut Vec<Self>, current_anchor_index: usize,
        ref_seq: &[u8], qry_seq: &[u8],
        penalties: &Penalties, cutoff: &Cutoff,
        is_hind_block: IsHindBlock,
    ) {
        // (1) DWFA
        let dwfa_res = {
            let current_anchor = &mut anchors[current_anchor_index];
            // 1. Get spare of penalty
            let (p_opp, l_opp) = if is_hind_block {
                if let AlignmentBlock::Estimated(p, l) = &current_anchor.fore {
                    (*p, *l)
                } else {
                    return; // if already aligned: pass
                }
            } else {
                if !current_anchor.dropped {
                    current_anchor.hind.penalty_and_length()
                } else {
                    return; // if dropped: pass
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
            // 2. DWFA
            if is_hind_block {
                dropout_wf_align(
                    &qry_seq[current_anchor.position.1+current_anchor.size..],
                    &ref_seq[current_anchor.position.0+current_anchor.size..],
                    penalty_spare,
                    penalties,
                    true,
                )
            } else {
                dropout_wf_align(
                    &qry_seq[..current_anchor.position.1],
                    &ref_seq[..current_anchor.position.0],
                    penalty_spare,
                    penalties,
                    false,
                )
            }
        };
        // (2) BACKTRACE
        match dwfa_res {
            Ok((wf, score, last_k)) => {
                // (1) get check points for backtrace
                let anchors_to_pass_check: AnchorsToPassCheck = Self::get_check_points_for_backtrace(
                    anchors, current_anchor_index, is_hind_block.clone()
                );
                // (2) bactrace
                let ((cigar, aligned_length), connected_backtraces) = dropout_wf_backtrace(
                    &wf, penalties, score, last_k, &anchors_to_pass_check
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
                        current_anchor.hind = AlignmentBlock::Extended(
                            ExactAlignment::Own(cigar, (score, aligned_length))
                        );
                    } else {
                        current_anchor.fore = AlignmentBlock::Extended(
                            ExactAlignment::Own(cigar, (score, aligned_length))
                        );
                    }
                    // update connected anchors
                    current_anchor.connected.extend(connected_anchor_indices.iter());
                }
                // (5) update connected anchors
                for (connected_anchor_index, (penalty_ref, length_ref)) in connected_backtraces {
                    let connected_anchor = &mut anchors[connected_anchor_index];
                    if is_hind_block {
                        // update anchor state
                        connected_anchor.hind = AlignmentBlock::Extended(
                            ExactAlignment::Ref(current_anchor_index, (score-penalty_ref, length_ref))
                        );
                        // update anchor's connected info
                        for check_point in &connected_anchor.check_points.1 {
                            if connected_anchor_indices.contains(check_point) {
                                connected_anchor.connected.insert(*check_point);
                            }
                        }
                    } else {
                        // update anchor state
                        connected_anchor.fore = AlignmentBlock::Extended(
                            ExactAlignment::Ref(current_anchor_index, (score-penalty_ref, length_ref))
                        );
                        // update anchor's connected info
                        for check_point in &connected_anchor.check_points.0 {
                            if connected_anchor_indices.contains(check_point) {
                                connected_anchor.connected.insert(*check_point);
                            }
                        };
                    }
                }
            },
            Err(_) => { // TODO: WF can be inherited to checkpoint
                anchors[current_anchor_index].drop();
            }
        }
    }
    #[inline]
    fn drop(&mut self) {
        self.dropped = true;
    }
    /*
    RESULTS
    */
    #[inline]
    fn drop_with_penalty_length(&mut self, cutoff: &Cutoff) -> Option<(Penalty, SequenceLength)> {
        let mut total_penalty: usize = 0;
        let mut total_length: usize = 0;
        // If already dropped: pass
        if self.dropped {
            return None;
        } else {
            // fore
            let (p, l) = self.fore.penalty_and_length();
            total_penalty += p;
            total_length += l;
            // anchor
            total_length += self.size;
            let (p, l) = self.hind.penalty_and_length();
            total_penalty += p;
            total_length += l;
        }
        // Evaluate
        if (total_length >= cutoff.ml) && (total_penalty as f64/total_length as f64 <= cutoff.ppl) {
            Some((total_penalty, total_length))
        } else {
            self.drop();
            None
        }
    }
    #[inline]
    fn clips_and_cigar(
        anchors: &Vec<Self>, current_anchor_index: usize,
        ref_len: SequenceLength, qry_len: SequenceLength
    ) -> (Clip, Clip, AlignedBlock) {
        let current_anchor = &anchors[current_anchor_index];
        // (1) Get cigar & ridx
        let (fore_cigar, fore_clip) = current_anchor.fore.get_cigar_ridx(
            anchors,
            FORE_BLOCK,
            current_anchor.position.0,
            current_anchor.position.1,
        );
        let (hind_cigar, hind_clip) = current_anchor.hind.get_cigar_ridx(
            anchors,
            HIND_BLOCK,
            ref_len-current_anchor.position.0-current_anchor.size,
            qry_len-current_anchor.position.1-current_anchor.size,
        );
        // (2) Generate empty new cigar
        let mut new_cigar: AlignedBlock = Vec::with_capacity(
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
    use crate::*;
    use super::*;

    #[test]
    fn test_new_anchor() {
        use crate::database::*;
        use crate::io::*;

        let ref_fasta = "./src/tests/fasta/ERR209055.fa";
        let qry_fasta = "./src/tests/fasta/ERR209056.fa";
        let qry_seq = {
            let mut qry_reader = fasta::fasta_records(qry_fasta);
            loop {
                let record = qry_reader.next().unwrap().unwrap();
                if record.id() == "k59_3595" {
                    break record.seq().to_vec();
                }
            }
        };
        
        let kmer_klt = 13;
        let ssr = 2;
        let ml = 100;
        let ppl = 0.1;
        let x = 4;
        let o = 6;
        let e = 2;

        let start = Instant::now();

        let cutoff = Cutoff::new(ml, ppl);
        let penalties = Penalties::new(x,o,e);

        let aligner = alignment::Aligner::new(cutoff, penalties);
        println!("kmer: {}", aligner.kmer);
        let (seq_provider, _) = sequence_provider::InMemorySequences::from_fasta(true, ref_fasta);

        let database_config = DatabaseConfig::new();

        let database = database_config.create_db(&seq_provider);
        let search_range = database.get_range();
        println!("db setted: {}", start.elapsed().as_nanos());

        let start = Instant::now();
        let mut anchors_group = AnchorsGroup::new(
            &database,
            &search_range,
            &aligner.cutoff,
            &aligner.block_penalty,
            aligner.kmer,
            &qry_seq
        );
        println!("AG setted: {}", start.elapsed().as_nanos());
        println!("{:#?}", anchors_group);

        let start = Instant::now();
        let res_for_db = anchors_group.alignment(
            &database,
            &aligner.penalties,
            &aligner.cutoff,
            &qry_seq,
            false
        );
        println!("Get res: {}", start.elapsed().as_nanos());

        println!("{:#?}", res_for_db);
    }
}