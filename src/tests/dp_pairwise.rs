use std::collections::HashSet;

use crate::alignment::{Alignment, BlockPenalty, Cutoff, calculate_kmer, Penalties};
use crate::io::cigar::{Cigar, Clip, Operation};

use bio::alignment::*;
use bio::alignment::AlignmentOperation;

use fm_index::converter::RangeConverter;
use fm_index::suffix_array::{SuffixOrderSampler};
use fm_index::FMIndex;
use fm_index::BackwardSearchIndex;

pub struct DpAligner {
    score_per_length: f64,
    kmer: usize,
    minimum_length: usize,
    mismatch_penalty: usize,
    gapopen_penalty: usize,
    gapext_penalty: usize,
}
type DpAnchor = (usize, usize, usize); // ref position, qry position, size

pub type DpResultCigar = Vec<Alignment>;
pub type DpResultVec = Vec<(Vec<AlignmentOperation>, usize)>;


impl DpAligner {
    pub fn new(
        score_per_length: f64, minimum_length: usize,
        mismatch_penalty: usize, gapopen_penalty: usize, gapext_penalty: usize
    ) -> Self {
        let kmer = calculate_kmer_dp(
            score_per_length, minimum_length, mismatch_penalty,
            gapopen_penalty, gapext_penalty
        );
        Self {
            score_per_length,
            kmer,
            minimum_length,
            mismatch_penalty,
            gapopen_penalty,
            gapext_penalty,
        }
    }
}

fn calculate_kmer_dp(
    score_per_length: f64, minimum_length: usize, mismatch_penalty: usize,
    gapopen_penalty: usize, gapext_penalty: usize
) -> usize {
    let bp = BlockPenalty::new(&Penalties::new(mismatch_penalty, gapopen_penalty, gapext_penalty ));
    let cutoff = Cutoff::new(minimum_length, score_per_length);
    let kmer = calculate_kmer(&cutoff, &bp);
    kmer
}

pub fn alignment(
    aligner: &DpAligner, ref_seq: &[u8], qry_seq: &[u8]
) -> Vec<Alignment> {
    // (1) get anchors (same as `anchorgroup`)
    let anchors = generate_anchors(aligner, ref_seq, qry_seq);
    // (2) alignment
    let mut result: DpResultVec = Vec::new();
    let mut used_operations: HashSet<Vec<AlignmentOperation>> = HashSet::new();
    for (ref_pos, qry_pos, size) in anchors {
        let mut operations: Vec<AlignmentOperation> = Vec::new();
        let mut penalty: usize = 0;
        // fore
        {
            let ref_seq: Vec<u8> = ref_seq[..ref_pos].iter().rev().map(|x| *x).collect();
            let qry_seq: Vec<u8> = qry_seq[..qry_pos].iter().rev().map(|x| *x).collect();
            let ref_is_long = ref_seq.len() >= qry_seq.len();
            let scoring = get_scoring(aligner.mismatch_penalty, aligner.gapopen_penalty, aligner.gapext_penalty, ref_is_long);
            let mut aligner = pairwise::Aligner::with_scoring(scoring);
            let alignment = aligner.custom(&qry_seq, &ref_seq);
            operations.extend(alignment.operations.iter().rev());
            penalty += -(alignment.score) as usize;
        }
        // match
        {
            operations.extend(vec![AlignmentOperation::Match; size]);
        }
        // hind
        {
            let ref_seq: Vec<u8> = ref_seq[ref_pos+size..].iter().map(|x| *x).collect();
            let qry_seq: Vec<u8> = qry_seq[qry_pos+size..].iter().map(|x| *x).collect();
            let ref_is_long = ref_seq.len() >= qry_seq.len();
            let scoring = get_scoring(aligner.mismatch_penalty, aligner.gapopen_penalty, aligner.gapext_penalty, ref_is_long);
            let mut aligner = pairwise::Aligner::with_scoring(scoring);
            let alignment = aligner.custom(&qry_seq, &ref_seq);
            operations.extend(alignment.operations.iter());
            penalty += -(alignment.score) as usize;
        }
        let length = operations.iter().filter(|op| {
            match op {
                AlignmentOperation::Xclip(_) | AlignmentOperation::Yclip(_) => false,
                _ => true,
            }
        }).count();
        if (length >= aligner.minimum_length) && (penalty as f64/length as f64 <= aligner.score_per_length) {
            if !used_operations.contains(&operations) {
                used_operations.insert(operations.clone());
                result.push((operations, penalty))
            }
        }
    };
    conv_to_cigar(result)
}

fn generate_anchors(
    aligner: &DpAligner, ref_seq: &[u8], qry_seq: &[u8],
) -> Vec<DpAnchor> {
    let index = {
        let seq: Vec<u8> = ref_seq.iter().cloned().collect();
        let converter = RangeConverter::new(b'A', b'T');
        let sampler = SuffixOrderSampler::new().level(2);
        FMIndex::new(seq, converter, sampler)
    };
    // push anchors
    let mut anchors: Vec<DpAnchor> = Vec::new();
    let qry_len = qry_seq.len();
    let search_count = qry_len / aligner.kmer;

    let mut pre_anchors: Vec<DpAnchor> = Vec::new();
    for i in 0..search_count {
        let qry_position = i*aligner.kmer;
        let pattern = &qry_seq[qry_position..qry_position+aligner.kmer];
        let search = index.search_backward(pattern);
        let positions = search.locate();

        let mut new_anchors: Vec<DpAnchor> = Vec::new();
        // generate new anchors
        for ref_pos in positions {
            if pre_anchors.len() == 0 {
                new_anchors.push((ref_pos as usize, i*aligner.kmer, aligner.kmer));
            } else {
                // Check Impeccable Extension
                pre_anchors = pre_anchors.into_iter().filter(|(ref_pos_pre, qry_pos_pre, size_pre)| {
                    if *ref_pos_pre + *size_pre == ref_pos as usize {
                        new_anchors.push((*ref_pos_pre, *qry_pos_pre, size_pre + aligner.kmer));
                        false
                    } else {
                        new_anchors.push((ref_pos as usize, i*aligner.kmer, aligner.kmer));
                        true
                    }
                }).collect();
            }
        };
        // extend & new -> pre
        anchors.extend(pre_anchors);
        pre_anchors = new_anchors;
    };
    anchors.extend(pre_anchors);
    anchors
}

fn get_scoring(mismatch_penalty: usize, gapopen_penalty: usize, gapext_penalty: usize, y_is_long: bool) -> pairwise::Scoring<pairwise::MatchParams> {
    let scoring = pairwise::Scoring::from_scores(
        -(gapopen_penalty as i32),
        -(gapext_penalty as i32),
        0,
        -(mismatch_penalty as i32),
    );
    if y_is_long {
        scoring.yclip_suffix(0)
    } else {
        scoring.xclip_suffix(0)
    }
}

fn conv_to_cigar(dp_result_vec: DpResultVec) -> Vec<Alignment> {
    dp_result_vec.into_iter().map(|(ops, penalty)| {
        let mut length = ops.len();
        let mut start_index: usize = 0;
        let mut end_index: usize = ops.len();
        let clip_front = {
            match ops[0] {
                AlignmentOperation::Xclip(x) => {
                    start_index += 1;
                    length -= 1;
                    Clip::Qry(x)
                },
                AlignmentOperation::Yclip(x) => {
                    start_index += 1;
                    length -= 1;
                    Clip::Ref(x)
                },
                _ => Clip::None
            }
        };
        let clip_end = {
            match ops[end_index-1] {
                AlignmentOperation::Xclip(x) => {
                    end_index -= 1;
                    length -= 1;
                    Clip::Qry(x)
                },
                AlignmentOperation::Yclip(x) => {
                    end_index -= 1;
                    length -= 1;
                    Clip::Ref(x)
                },
                _ => Clip::None
            }
        };
        let mut cigar: Cigar = Vec::new();
        for op in ops[start_index..end_index].iter() {
            let next_cigar_op = match op {
                AlignmentOperation::Match => Operation::Match,
                AlignmentOperation::Subst => Operation::Subst,
                AlignmentOperation::Del => Operation::Del,
                AlignmentOperation::Ins => Operation::Ins,
                _ => panic!("op conv err"),
            };
            
            let elong = match cigar.last() {
                Some((last_op, _)) => {
                    if *last_op == next_cigar_op {
                        true
                    } else {
                        false
                    }
                },
                None => false
            };

            if elong {
                cigar.last_mut().unwrap().1 += 1;
            } else {
                cigar.push((next_cigar_op, 1));
            }
        };
        Alignment {
            length,
            penalty,
            clip_front,
            clip_end,
            cigar,
        }
    }).collect()
}