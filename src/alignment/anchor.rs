use crate::{SequenceLength, Penalty};
use crate::io::cigar::{
    Cigar, Operation, Clip, ReverseIndex,
    get_reverse_index_from_own, get_reverse_index_from_ref,
};
use super::{Cutoff, Penalties, BlockPenalty, FmIndex, Alignment, AlignmentResult};
use super::dwfa::{
    WaveFront, AnchorsToPassCheck, CigarReference, BacktraceResult, RefToBacktrace,
    dropout_wf_align, dropout_wf_backtrace
};


use std::time::{Duration, Instant};


use core::panic;
use std::cmp::{min, max};
use std::collections::{HashMap, HashSet};
use std::iter::FromIterator;

type IsHindBlock = bool;
const HIND_BLOCK: bool = true;
const FORE_BLOCK: bool = false;


use crate::reference::Reference;

// Anchor Group
pub struct AnchorsGroup {
    anchors_by_seq: Vec<AnchorsRecord>,
}
impl AnchorsGroup {
    #[inline]
    pub fn alignment_with_anchor(
        penalties: &Penalties,
        cutoff: &Cutoff,
        block_penalty: &BlockPenalty,
        reference: &Reference,
        kmer: usize,
        query: &[u8],
        get_minimum_penalty: bool,
    ) -> AlignmentResult {
        let mut ag = Self::new(penalties, cutoff, block_penalty, reference, kmer, query);
        ag.alignment(penalties, cutoff, reference, query, get_minimum_penalty)
    }
    #[inline]
    pub fn new(
        penalties: &Penalties,
        cutoff: &Cutoff,
        block_penalty: &BlockPenalty,
        reference: &Reference,
        kmer: usize,
        query: &[u8],
    ) -> Self {
        #[cfg(test)]
        let start_time = Instant::now();
        // TODO: in case just one record 
        let records_count: usize = reference.accumulated_length.len();
        let mut preset_container: Vec<AnchorsPreset> = vec![AnchorsPreset::new(); records_count];

        let qry_len = query.len();
        let search_count = qry_len / kmer;
        let mut anchor_existence: Vec<bool> = Vec::with_capacity(search_count);
        /*
        (1) Create Preset of Anchors
        */
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
            for record_idx in 0..records_count {
                // Get AnchorsPreset
                let anchors_preset = &mut preset_container[record_idx];
                // Make positions to add
                let mut curr_positions: Vec<AnchorPosition> = Vec::new();
                let mut ie_pre_positions: Vec<AnchorPosition> = Vec::new();
                // Fetch reference information
                let ref_last_index = &reference.accumulated_length[record_idx];
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
        });
        #[cfg(test)]
        println!("1,new_preset,{}", start_time.elapsed().as_nanos()); let start_time = Instant::now();
        /*
        (2) Preset to new anchors
        */
        // Convert existence info to block penalty
        let (forward_bp, backward_bp) = existence_to_bp(anchor_existence, block_penalty);
        // AnchorsPreset to new anchors
        let mut ref_start_pos = 0;
        let mut ref_end_pos = 0;
        let mut anchors_by_seq: Vec<AnchorsRecord> = { // : Vec<Vec<Anchor>>
            preset_container.into_iter().enumerate().map(|(record_index, anchors_preset)| {
                ref_end_pos = reference.accumulated_length[record_index] as usize;
                let anchors = anchors_preset.create_anchors(
                    &kmer,
                    &qry_len,
                    &forward_bp,
                    &backward_bp,
                    &ref_start_pos,
                    &ref_end_pos,
                    cutoff
                );
                ref_start_pos = ref_end_pos;
                anchors
            }).collect()
        };
        #[cfg(test)]
        println!("2,preset_to_new,{}", start_time.elapsed().as_nanos());
        #[cfg(test)]
        {
            let len: usize = anchors_by_seq.iter().map(|x| x.len()).sum();
            println!("#anchorcount:{}", len);
        }
        #[cfg(test)]
        let start_time = Instant::now();
        /*
        (3) Generate checkpoints
        */
        anchors_by_seq.iter_mut().for_each(|anchors_record| {
            Anchor::create_check_points(anchors_record, penalties, cutoff);
        });
        #[cfg(test)]
        println!("3,gen_chkp,{}", start_time.elapsed().as_nanos()); let start_time = Instant::now();
        Self {
            anchors_by_seq: anchors_by_seq,
        }
    }
    #[inline]
    pub fn alignment(
        &mut self,
        penalties: &Penalties,
        cutoff: &Cutoff,
        reference: &Reference,
        query: &[u8],
        get_minimum_penalty: bool,
    ) -> AlignmentResult {
        type AlignmentPreset = HashMap<usize, (usize, usize)>; // anchor index, (penalty, length)

        let mut result: AlignmentResult = Vec::new();
        let mut minimum_penalty: Penalty = Penalty::MAX;

        let seq_records = &reference.sequence_records;
        seq_records.iter().enumerate().zip(self.anchors_by_seq.iter_mut()).for_each(|((rec_idx, seq_rec), anchors)| {
            #[cfg(test)]
            let start_time = Instant::now();
            // (1) Alignment hind
            for idx in 0..anchors.len() {
                Anchor::alignment(
                    anchors,
                    idx,
                    &seq_rec.sequence,
                    query,
                    penalties,
                    cutoff,
                    HIND_BLOCK,
                );
            }
            #[cfg(test)]
            println!("4,hind,{}", start_time.elapsed().as_nanos()); let start_time = Instant::now();
            // (2) Alignment fore
            for idx in (0..anchors.len()).rev() {
                Anchor::alignment(
                    anchors,
                    idx,
                    &seq_rec.sequence,
                    query,
                    penalties,
                    cutoff,
                    FORE_BLOCK,
                );
            }
            println!("5,fore,{}", start_time.elapsed().as_nanos()); let start_time = Instant::now();
            // (3) Get AlignmentPreset of valid anchors
            let alignment_preset: AlignmentPreset = if get_minimum_penalty {
                
                let mut anchors_map: AlignmentPreset = HashMap::new();
                for (anchor_index, anchor) in anchors.iter_mut().enumerate() {
                    if let Some((penalty, length)) = anchor.drop_with_penalty_length(cutoff) {
                        if penalty < minimum_penalty {
                            minimum_penalty = penalty;
                            result.clear();
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
            println!("6,preset,{}", start_time.elapsed().as_nanos()); let start_time = Instant::now();
            // (4) Get unique anchors
            let unique_anchors = unique_symbols_filtering(valid_anchors, &*anchors);
            // (5) Get cigar & penalty
            let ref_len = seq_rec.sequence.len();
            let qry_len = query.len();
            let res: Vec<Alignment> = unique_anchors.into_iter().map(|unique_anchor_index| {
                let (clip_front, clip_end, cigar) = Anchor::clips_and_cigar(
                    anchors,
                    unique_anchor_index,
                    ref_len,
                    qry_len
                ); // FIXME: can be ref to dropped
                let (penalty, length) = alignment_preset.get(&unique_anchor_index).unwrap();
                Alignment {
                    penalty: *penalty,
                    length: *length,
                    clip_front: clip_front, 
                    clip_end: clip_end,
                    cigar: cigar, 
                }
            }).collect();
            println!("7,get_unique,{}", start_time.elapsed().as_nanos()); let start_time = Instant::now();
            if res.len() != 0 {
                for a in res {
                    result.push((rec_idx, a));
                }
            }
        });
        result
    }
}

/*

ANCHOR

*/
type AnchorsRecord = Vec<Anchor>; // Created from [AnchorsPreset]

#[derive(Debug, Clone)]
struct Anchor { 
    position: (usize, usize), // ref, qry
    size: usize,
    state: AlignmentState,
    check_points: (Vec<usize>, Vec<usize>), // fore, hind
    connected: HashSet<usize>,
}
#[derive(Debug, Clone)]
enum AlignmentState {
    Dropped,
    Aligned(AlignmentBlock, AlignmentBlock), // fore, hind
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
}
#[derive(Debug, Clone)]
enum ExactAlignment {
    Own(Cigar, (usize, usize)), // Cigar, (penalty, length)
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
    ) -> (Option<(&'a Cigar, usize, u32)>, Clip) { // (cigar, cigar length, offset), Clip
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
                    if let AlignmentState::Aligned(_, AlignmentBlock::Extended(
                        ExactAlignment::Own(cigar, _)
                    )) = &anchors[*ref_anchor_index].state {
                        cigar
                    } else {
                        // TODO: err msg
                        panic!("Trying to get result operations from invalid anchor.");
                    }
                } else {
                    if let AlignmentState::Aligned(AlignmentBlock::Extended(
                        ExactAlignment::Own(cigar, _)
                    ), _) = &anchors[*ref_anchor_index].state {
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
    fn create_check_points(anchors_record: &mut AnchorsRecord, penalties: &Penalties, cutoff: &Cutoff) {
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
            if let AlignmentState::Aligned(AlignmentBlock::Estimated(p, l), _) = &fore.state {
                penalty += p;
                length += l;
            }
            // hind
            if let AlignmentState::Aligned(_, AlignmentBlock::Estimated(p, l)) = &hind.state {
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
                if let AlignmentState::Aligned(
                    AlignmentBlock::Estimated(p, l),
                    AlignmentBlock::Estimated(_, _),
                ) = &current_anchor.state {
                    (*p, *l)
                } else {
                    return; // if already aligned: pass
                }
            } else {
                if let AlignmentState::Aligned(
                    AlignmentBlock::Estimated(_, _),
                    extended_block
                ) = &current_anchor.state {
                    extended_block.penalty_and_length()
                } else {
                    return; // if already aligned: pass
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
                        if let AlignmentState::Aligned(_, alignment_block) = &mut current_anchor.state {
                            *alignment_block = AlignmentBlock::Extended(
                                ExactAlignment::Own(cigar, (score, aligned_length))
                            );
                        }
                    } else {
                        if let AlignmentState::Aligned(alignment_block, _) = &mut current_anchor.state {
                            *alignment_block = AlignmentBlock::Extended(
                                ExactAlignment::Own(cigar, (score, aligned_length))
                            );
                        }
                    }
                    // update connected anchors
                    current_anchor.connected.extend(connected_anchor_indices.iter());
                }
                // (5) update connected anchors
                for (connected_anchor_index, (penalty_ref, length_ref)) in connected_backtraces {
                    let connected_anchor = &mut anchors[connected_anchor_index];
                    if is_hind_block {
                        // update anchor state
                        if let AlignmentState::Aligned(_, alignment_block) = &mut connected_anchor.state {
                            *alignment_block = AlignmentBlock::Extended(
                                ExactAlignment::Ref(current_anchor_index, (score-penalty_ref, length_ref))
                            );
                        }
                        // update anchor's connected info
                        for check_point in &connected_anchor.check_points.1 {
                            if connected_anchor_indices.contains(check_point) {
                                connected_anchor.connected.insert(*check_point);
                            }
                        }
                    } else {
                        // update anchor state
                        if let AlignmentState::Aligned(alignment_block, _) = &mut connected_anchor.state {
                            *alignment_block = AlignmentBlock::Extended(
                                ExactAlignment::Ref(current_anchor_index, (score-penalty_ref, length_ref))
                            );
                        }
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
        self.state = AlignmentState::Dropped;
    }
    /*
    RESULTS
    */
    #[inline]
    fn drop_with_penalty_length(&mut self, cutoff: &Cutoff) -> Option<(Penalty, SequenceLength)> {
        let mut total_penalty: usize = 0;
        let mut total_length: usize = 0;
        // If already dropped: pass
        if let AlignmentState::Aligned(fore, hind) = &self.state {
            // fore
            let (p, l) = fore.penalty_and_length();
            total_penalty += p;
            total_length += l;
            // anchor
            total_length += self.size;
            // hind
            let (p, l) = hind.penalty_and_length();
            total_penalty += p;
            total_length += l;
        } else {
            return None;
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
    ) -> (Clip, Clip, Cigar) {
        let current_anchor = &anchors[current_anchor_index];
        if let AlignmentState::Aligned(
            AlignmentBlock::Extended(fore),
            AlignmentBlock::Extended(hind),
        ) = &current_anchor.state {
            // (1) Get cigar & ridx
            let (fore_cigar, fore_clip) = fore.get_cigar_ridx(
                anchors,
                FORE_BLOCK,
                current_anchor.position.0,
                current_anchor.position.1,
            );
            let (hind_cigar, hind_clip) = hind.get_cigar_ridx(
                anchors,
                HIND_BLOCK,
                ref_len-current_anchor.position.0-current_anchor.size,
                qry_len-current_anchor.position.1-current_anchor.size,
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

/*

ANCHOR PRESET

*/
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
    fn create_anchors(
        self,
        kmer: &usize,
        qry_len: &usize,
        forward_bp: &Vec<usize>,
        backward_bp: &Vec<usize>,
        ref_start_pos: &usize,
        ref_end_pos: &usize,
        cutoff: &Cutoff,
    ) -> Vec<Anchor> {
        self.pushed_positions.into_iter().filter_map(|anchor_position| {
            let ref_pos = anchor_position.ref_pos - ref_start_pos;
            let qry_idx = anchor_position.qry_idx;
            let qry_pos = qry_idx * kmer;
            let size = anchor_position.size;
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
                let ref_left = ref_end_pos - anchor_position.ref_pos;
                let qry_left = qry_len - qry_pos;
                let mut length = min(ref_left, qry_left);
                let block_count = length / kmer;
                let penalty: usize = forward_bp[qry_idx..qry_idx+block_count].iter().map(|p| {
                    if *p != 0 { length += 1; };
                    *p
                }).sum();
                total_length += length;
                total_penalty += penalty;
                AlignmentBlock::Estimated(penalty, length-size)
            };
            if (total_length >= cutoff.ml) && (total_penalty as f64/total_length as f64) <= cutoff.ppl {
                Some(
                    Anchor {
                        position: (ref_pos, qry_pos), // ref, qry
                        size: size,
                        state: AlignmentState::Aligned(fore_block, hind_block),
                        check_points: (Vec::new(), Vec::new()), // fore, hind
                        connected: HashSet::new(),
                    }
                )
            } else {
                None
            }
        }).collect()
    }
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
    use crate::reference::*;
    use crate::alignment::*;
    use crate::io::*;
    use super::*;

    use std::time::{Duration, Instant};

    #[test]
    fn test_with_two_files() {

        let ref_fasta = "./src/tests/fasta/ERR209055.fa";
        let qry_fasta = "./src/tests/fasta/ERR209056.fa";
        
        let kmer_klt = 13;
        let ssr = 2;
        let ml = 100;
        let ppl = 0.1;
        let x = 4;
        let o = 6;
        let e = 2;

        let now = Instant::now();

        let cutoff = Cutoff::new(ml, ppl);
        let penalties = Penalties::new(x,o,e);
        let reference = ReferenceConfig::new()
            .contain_only_nucleotide(true)
            .search_reverse_complement(true)
            .set_kmer_size_for_klt(kmer_klt)
            .set_sampling_ratio_for_sa(ssr)
            .generate_reference_with_fasta_file(ref_fasta);
        
        let aligner = Aligner::new(cutoff, penalties, reference);

        println!("0,set_aligner,{}", now.elapsed().as_nanos());

        println!("#kmer: {:?}", aligner.kmer);
        let mut qry_reader = fasta::fasta_records(qry_fasta);
        while let Some(Ok(record)) = qry_reader.next() {
            println!("#{}", record.id());
            let res = aligner.alignment_with_sequence(
                record.seq(),
                false,
            );
            println!("#{:?}", res);
        };
    }
}