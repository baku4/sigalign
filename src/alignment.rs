//! Dropout alignment core
mod anchor;
mod dwfa;

use std::fmt::Debug;

use crate::{SequenceLength, Penalty};
use crate::io::Alignment;
use crate::reference::{Reference, FmIndex};

// Alignment Result
pub type AlignmentResult = Vec<Alignment>;

#[derive(Debug, Clone)]
pub struct Penalties {
    x: Penalty,
    o: Penalty,
    e: Penalty,
}
impl Penalties {
    /// Generate new [Penalties] which is required to [Aligner]
    /// - x: penalty for mismatch
    /// - o: penalty for gap opening
    /// - e: penalty for gap extending
    pub fn new(x: usize, o: usize, e: usize) -> Self {
        Self { x, o, e }
    }
}

#[derive(Debug, Clone)]
pub struct Cutoff {
    ml: SequenceLength,
    ppl: f64,
}
impl Cutoff {
    /// Generate new [Cutoff] which is required to [Aligner]
    /// - ml: minimum aligned length
    /// - ppl: penalty per aligned length
    pub fn new(ml: SequenceLength, ppl: f64) -> Self {
        Self{ ml, ppl }
    }
}

pub struct Aligner {
    cutoff: Cutoff,
    penalties: Penalties,
    // Auto caluclated
    block_penalty: BlockPenalty,
    kmer: usize,
    // Options
    using_cached_wf: bool,
    get_minimum_penalty: bool,
    // Reference
    reference: Option<Reference>,
}

impl Aligner {
    pub fn new(cutoff: Cutoff, penalties: Penalties) -> Self {
        let block_penalty = BlockPenalty::new(&penalties);
        let kmer = calculate_kmer(&cutoff, &block_penalty);
        Self {
            cutoff,
            penalties,
            block_penalty,
            kmer,
            // Options
            using_cached_wf: false,
            get_minimum_penalty: false,
            reference: None,
        }
    }
    pub fn get_minimum_penalty(mut self) -> Self {
        self.get_minimum_penalty = true;
        self
    }
    pub fn load_reference(&mut self) {
        //
    }
    pub fn clear_reference(&mut self) {
        //
    }
    pub fn change_reference(&mut self) {
        //
    }
    pub fn alignment(&mut self) {
        //
    }
    /*
    pub fn align_with_only_sequences(&self, ref_seq: &[u8] , qry_seq: &[u8]) -> Option<AlignmentResult> {
        let index = ReferenceDep::fmindex(&ref_seq);
        let result = match anchor::AnchorGroup::new(
            ref_seq, qry_seq, &index, self.kmer,
            &self.block_penalty, &self.penalties, &self.cutoff
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
    */
}

#[derive(Debug, Clone)]
pub struct BlockPenalty {
    odd: Penalty,
    even: Penalty,
}

impl BlockPenalty {
    pub fn new(penalties: &Penalties) -> Self {
        let mo: Penalty;
        let me: Penalty;
        if penalties.x <= penalties.o + penalties.e {
            mo = penalties.x;
            if penalties.x * 2 <= penalties.o + (penalties.e * 2) {
                me = penalties.x;
            } else {
                me = penalties.o + (penalties.e * 2) - penalties.x;
            }
        } else {
            mo = penalties.o + penalties.e;
            me = penalties.e;
        }
        Self { odd: mo, even: me }
    }
}

pub fn calculate_kmer(cutoff: &Cutoff, block_penalty: &BlockPenalty) -> usize {
    let mut i: usize = 1;
    let mut kmer: f64;
    loop {
        kmer = (((cutoff.ml+2) as f64/(2*i) as f64) - 1_f64).ceil();
        if (i*(block_penalty.odd + block_penalty.even)) as f64 > cutoff.ppl * 2_f64 * (((i+1) as f64)*kmer-1_f64) {
            break;
        } else {
            i += 1;
        }
    }
    kmer as usize
}