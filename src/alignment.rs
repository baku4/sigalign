//! Dropout alignment core
mod anchor;
mod dwfa;

use crate::{SequenceLength, OperationLength, Penalty};
use crate::io::Alignment;
use crate::io::cigar::{Cigar, Clip};

use lt_fm_index::Config as FmConfig;
use lt_fm_index::FmIndex;

const FM_KLT_KMER_SIZE: usize = 8;
const FM_SA_SAMPLING_RATIO: u64 = 2;


#[derive(Debug)]
pub struct Aligner {
    cutoff: Cutoff,
    kmer: usize,
    scores: Scores,
    // TODO: emp kmer naming
    emp_kmer: BlockPenalty,
    using_cached_wf: bool,
    get_minimum_penalty: bool,
}

// Alignment Result: (operations, penalty)
type AlignmentResultForDep = Vec<(Vec<Operation>, Penalty)>;
// Alignment Result
pub type AlignmentResult = Vec<Alignment>;

impl Aligner {
    pub fn new(
        score_per_length: f64,
        minimum_length: usize,
        mismatch_penalty: usize,
        gapopen_penalty: usize,
        gapext_penalty: usize,
        using_cached_wf: bool,
        get_minimum_penalty: bool
    ) -> Self {
        let emp_kmer = BlockPenalty::new(mismatch_penalty, gapopen_penalty, gapext_penalty);
        let kmer = Self::kmer_calculation(score_per_length, minimum_length, &emp_kmer);
        Self {
            cutoff: Cutoff {
                score_per_length: score_per_length,
                minimum_length: minimum_length,
            },
            kmer: kmer,
            scores: (mismatch_penalty, gapopen_penalty, gapext_penalty),
            emp_kmer: emp_kmer,
            using_cached_wf: using_cached_wf,
            get_minimum_penalty: get_minimum_penalty,
        }
    }
    pub fn kmer_calculation(score_per_length: f64, minimum_length: usize, emp_kmer: &BlockPenalty) -> usize {
        let mut i: usize = 1;
        let mut kmer_size: f64;
        loop {
            kmer_size = (((minimum_length+2) as f64/(2*i) as f64) - 1_f64).ceil();
            if (i*(emp_kmer.odd + emp_kmer.even)) as f64 > score_per_length * 2_f64 * (((i+1) as f64)*kmer_size-1_f64) {
                break;
            } else {
                i += 1;
            }
        }
        kmer_size as usize
    }
    pub fn perform_with_sequence_using_new_anchor(&self, ref_seq: &[u8] , qry_seq: &[u8]) -> Option<AlignmentResult> {
        let penalties = Penalties {
            x: self.scores.0,
            o: self.scores.1,
            e: self.scores.2,
        };
        let index = Reference::fmindex(&ref_seq);
        let result = match anchor::AnchorGroup::new(
            ref_seq, qry_seq, &index, self.kmer,
            &self.emp_kmer, &penalties, &self.cutoff
        ) {
            Some(mut anchor_group) => {
                anchor_group.alignment(self.using_cached_wf);
                let result = anchor_group.get_result(self.get_minimum_penalty);
                if result.len() == 0 {
                    None
                } else {
                    Some(result)
                }
            },
            None => None,
        };
        result
    }
}

pub struct Reference<T: AsRef<[u8]>>{
    sequence: T,
    index: FmIndex
}
impl<T: AsRef<[u8]>> Reference<T> {
    pub fn new(sequence: T) -> Self {
        let fm_index =  Self::fmindex(&sequence);
        Self {
            sequence: sequence,
            index: fm_index,
        }
    }
    fn fmindex(sequence: &T) -> FmIndex {
        let seq = sequence.as_ref().iter().cloned().collect();
        // TODO: Custom fmindex configuration
        let fm_config: FmConfig = FmConfig::new()
            .set_kmer_lookup_table(FM_KLT_KMER_SIZE)
            .set_suffix_array_sampling_ratio(FM_SA_SAMPLING_RATIO);
        FmIndex::new(&fm_config, seq)
    }
}

type Scores = (usize, usize, usize);

#[derive(Debug)]
pub struct Penalties {
    x: Penalty,
    o: Penalty,
    e: Penalty,
}

#[derive(Debug)]
pub struct Cutoff {
    score_per_length: f64,
    // TODO: min length to u64
    minimum_length: usize,
}

#[derive(Debug)]
pub struct BlockPenalty {
    odd: usize,
    even: usize,
}

impl BlockPenalty {
    pub fn new(mismatch_penalty: usize, gapopen_penalty: usize, gapext_penalty: usize) -> Self {
        let mo: usize;
        let me: usize;
        if mismatch_penalty <= gapopen_penalty + gapext_penalty {
            mo = mismatch_penalty;
            if mismatch_penalty * 2 <= gapopen_penalty + (gapext_penalty * 2) {
                me = mismatch_penalty;
            } else {
                me = gapopen_penalty + (gapext_penalty * 2) - mismatch_penalty;
            }
        } else {
            mo = gapopen_penalty + gapext_penalty;
            me = gapext_penalty;
        }
        Self {
            odd: mo,
            even: me,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Operation {
    Match,
    Subst,
    Ins,
    Del,
    RefClip(u64),
    QryClip(u64),
}
