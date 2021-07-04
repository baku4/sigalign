use crate::alignment::*;
use crate::alignment::anchor::*;

use bio::alignment::*;
use bio::alignment::AlignmentOperation;

use fm_index::converter::RangeConverter;
use fm_index::suffix_array::{SuffixOrderSampledArray, SuffixOrderSampler};
use fm_index::FMIndex;
use fm_index::BackwardSearchIndex;

/// ref position, qry position, size
type DPanchor = (usize, usize, usize);
type DPresult = Vec<(Vec<AlignmentOperation>, usize)>;

pub fn get_kmer(score_per_length: f64, minimum_length: usize, mismatch_penalty: usize, gapopen_penalty: usize, gapext_penalty: usize) -> usize {
    let kmer = Aligner::kmer_calculation(score_per_length, minimum_length, &EmpKmer::new(mismatch_penalty, gapopen_penalty, gapext_penalty));
    kmer
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
pub fn alignment(ref_seq: &[u8], qry_seq: &[u8],
    score_per_length: f64, minimum_length: usize, kmer: usize,
    mismatch_penalty: usize, gapopen_penalty: usize, gapext_penalty: usize) -> DPresult {
    // (1) get anchors (same as `anchorgroup`)
    let anchors = {
        let index = {
            let seq: Vec<u8> = ref_seq.iter().cloned().collect();
            let converter = RangeConverter::new(b'A', b'T');
            let sampler = SuffixOrderSampler::new().level(2);
            FMIndex::new(seq, converter, sampler)
        };
        // push anchors
        let mut anchors: Vec<DPanchor> = Vec::new();
        let qry_len = qry_seq.len();
        let search_count = qry_len / kmer;

        let mut pre_anchors: Vec<DPanchor> = Vec::new();
        for i in 0..search_count {
            let qry_position = i*kmer;
            let pattern = &qry_seq[qry_position..qry_position+kmer];
            let search = index.search_backward(pattern);
            let positions = search.locate();

            let mut new_anchors: Vec<DPanchor> = Vec::new();
            // generate new anchors
            for ref_pos in positions {
                if pre_anchors.len() == 0 {
                    new_anchors.push((ref_pos as usize, i*kmer, kmer));
                } else {
                    // Check Impeccable Extension
                    for (pre_idx, (ref_pos_pre, qry_pos_pre, size_pre)) in pre_anchors.clone().into_iter().enumerate() {
                        if ref_pos_pre + size_pre == ref_pos as usize {
                            pre_anchors.remove(pre_idx);
                            new_anchors.push((ref_pos_pre, qry_pos_pre, size_pre + kmer));
                        } else {
                            new_anchors.push((ref_pos as usize, i*kmer, kmer));
                        }
                    }
                }
            };
            // extend & new -> pre
            anchors.extend(pre_anchors);
            pre_anchors = new_anchors;
        };
        anchors.extend(pre_anchors);
        anchors
    };
    // (2) alignment
    let mut result: DPresult = Vec::new();
    for (ref_pos, qry_pos, size) in anchors {
        let mut operations: Vec<AlignmentOperation> = Vec::new();
        let mut penalty: usize = 0;
        // fore
        {
            let ref_seq: Vec<u8> = ref_seq[..ref_pos].iter().rev().map(|x| *x).collect();
            let qry_seq: Vec<u8> = qry_seq[..qry_pos].iter().rev().map(|x| *x).collect();
            let ref_is_long = ref_seq.len() >= qry_seq.len();
            let scoring = get_scoring(mismatch_penalty, gapopen_penalty, gapext_penalty, ref_is_long);
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
            let scoring = get_scoring(mismatch_penalty, gapopen_penalty, gapext_penalty, ref_is_long);
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
        if (length >= minimum_length) && (penalty as f64/length as f64 <= score_per_length) {
            result.push((operations, penalty))
        }
    };
    result
}
pub fn get_aligner(score_per_length: f64, minimum_length: usize, mismatch_penalty: usize, gapopen_penalty: usize, gapext_penalty: usize) -> Aligner {
    Aligner::new(
        score_per_length,
        minimum_length,
        mismatch_penalty,
        gapopen_penalty,
        gapext_penalty,
        false,
        false,
    )
}
pub fn alignment_with_anchorgroup(aligner: &Aligner, ref_seq: &[u8] , qry_seq: &[u8]) {

}