//! Dropout alignment core
mod dwfa;
pub mod anchor;
pub mod anchor_dep;

use std::fmt::Debug;

use crate::{SequenceLength, Penalty};
use crate::reference::{Reference, FmIndex};
use crate::io::cigar;
use anchor::AnchorsGroup;

// Alignment Result
#[derive(Debug)]
pub struct Alignment {
    pub penalty: Penalty,
    pub length: SequenceLength,
    pub clip_front: cigar::Clip,
    pub clip_end: cigar::Clip,
    pub cigar: cigar::Cigar,
}
pub type AlignmentResult = Vec<(usize, Alignment)>; // (Index of reference, Alignment)


pub type AlignmentResultDep = Vec<(Alignment)>; // TODO: to dep

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
    pub cutoff: Cutoff,
    pub penalties: Penalties,
    // Auto caluclated
    pub block_penalty: BlockPenalty,
    pub kmer: usize,
    // Options
    pub using_cached_wf: bool,
    pub get_minimum_penalty: bool,
    // Reference
    pub reference: Reference,
}

impl Aligner {
    /// Create new aligner
    pub fn new(cutoff: Cutoff, penalties: Penalties, reference: Reference) -> Self {
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
            reference: reference,
        }
    }
    /// Change alignment options
    pub fn get_minimum_penalty(mut self) -> Self {
        self.get_minimum_penalty = true; self
    }
    pub fn change_reference(&mut self, reference: Reference) {
        self.reference = reference;
    }
    /// Alignment
    #[inline]
    pub fn alignment_with_sequence(&self, query: &[u8], get_minimum_penalty: bool) -> AlignmentResult {
        let alignment_res = AnchorsGroup::alignment_with_anchor(
            &self.penalties,
            &self.cutoff,
            &self.block_penalty,
            &self.reference,
            self.kmer,
            query,
            get_minimum_penalty,
        );
        alignment_res
    }
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