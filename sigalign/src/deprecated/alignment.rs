pub mod operation;
mod anchor;
mod dwfa;

use crate::deprecated::{Penalty, SequenceLength, database::{Database, SearchRange}};

/// Result of alignment
#[derive(Debug)]
pub struct AlignmentResult {
    pub penalty: Penalty,
    pub length: SequenceLength,
    pub clip_front: operation::Clip,
    pub clip_end: operation::Clip,
    pub aligned_block: operation::Operations,
}

/// Result of alignment for database
pub type AlignmentResultByDbIndex = std::collections::HashMap<usize, Vec<AlignmentResult>>;

/// Scoring scheme for alignment
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

/// Cutoff for alignment
#[derive(Debug, Clone)]
pub struct Cutoff {
    ml: SequenceLength,
    ppl: f64,
}
impl Cutoff {
    /// Generate new [Cutoff] which is required to [Aligner]
    /// - ml: minimum aligned length
    /// - ppl: penalty per aligned length
    #[inline]
    pub fn new(ml: SequenceLength, ppl: f64) -> Self {
        Self{ ml, ppl }
    }
}

/// Aligner
pub struct Aligner {
    pub cutoff: Cutoff,
    pub penalties: Penalties,
    // Auto caluclated
    pub block_penalty: BlockPenalty,
    pub kmer: usize,
    // Options
    pub using_cached_wf: bool,
    pub get_minimum_penalty: bool,
}

impl Aligner {
    /// Create new aligner
    #[inline]
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
        }
    }
    /// Change alignment options
    #[inline]
    pub fn get_minimum_penalty(mut self) -> Self {
        self.get_minimum_penalty = true; self
    }
    /// Alignment with query sequence
    #[inline]
    pub fn alignment_with_query(&self, database: &Database, search_range: &SearchRange, query: &[u8], get_minimum_penalty: bool) -> AlignmentResultByDbIndex {
        let mut anchors_group = anchor::AnchorsGroup::new(
            database,
            search_range,
            &self.cutoff,
            &self.block_penalty,
            self.kmer,
            query,
        );
        anchors_group.alignment(
            database,
            &self.penalties,
            &self.cutoff,
            query,
            get_minimum_penalty
        )
    }
}

#[derive(Debug, Clone)]
pub struct BlockPenalty {
    odd: Penalty,
    even: Penalty,
}

impl BlockPenalty {
    #[inline]
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
#[inline]
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
