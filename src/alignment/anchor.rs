use std::cmp::min;

use super::{FmIndex, Operation, EmpKmer};
use fm_index::BackwardSearchIndex;

struct AnchorGroup<'a> {
    ref_seq: &'a [u8],
    qry_seq: &'a [u8],
    kmer: usize,
    emp_kmer: &'a EmpKmer,
    anchors: Vec<Anchor>,
}
impl<'a> AnchorGroup<'a> {
    fn new(ref_seq: &[u8], qry_seq: &[u8], index: &FmIndex, kmer: usize, emp_kmer: &EmpKmer) { // -> Self
        let ref_len = ref_seq.len();
        let qry_len = qry_seq.len();
        let search_count = qry_len / kmer;
        let mut anchors_preset: Vec<Anchor> = Vec::new();
        let mut anchor_existence: Vec<bool> = Vec::with_capacity(search_count+1); // first value is buffer
        // (1) Generate Anchors Proto
        {
            let mut anchors_cache: Option<Vec<Anchor>> = None;
            for i in 0..search_count {
                let qry_position = i*kmer;
                let pattern = &qry_seq[qry_position..qry_position+kmer];
                let search = index.search_backward(pattern);
                let positions = search.locate();
                // ** Check Impeccable Extension **
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
        }
        // (2) Calculate the EMP values
        anchors_preset.iter_mut().for_each(|anchor| {
            anchor.to_raw_state(ref_len, qry_len, kmer, &anchor_existence, &emp_kmer);
        });
        // (3) Set up checkpoints

        ()
    }
    fn is_empty() {

    }
}

struct Anchor {
    position: (usize, usize), // (ref, qry)
    size: usize,
    state: AnchorState,
    check_points: (Vec<usize>, Vec<usize>), // (fore, hind)
}

impl Anchor {
    fn new(ref_pos: usize, qry_pos: usize, kmer: usize) -> Self {
        Self {
            position: (ref_pos, qry_pos),
            size: kmer,
            state: AnchorState::Proto,
            check_points: (Vec::new(), Vec::new()),
        }
    }
    fn impeccable_extension(mut self, kmer: usize) -> Self {
        self.size += kmer;
        self
    }
    fn to_raw_state(&mut self, ref_len: usize, qry_len: usize, kmer: usize, anchor_existence: &Vec<bool>, emp_kmer: &EmpKmer) {
        let block_index = self.position.1 / qry_len;
        // fore block
        let fore_emp_block = {
            let block_len = min(self.position.0, self.position.1);
            let quot = block_len / kmer;
            let rem = block_len % kmer;
            let mut odd_block_count: usize = 0;
            let mut even_block_count: usize = 0;
            let mut previous_block_is_odd = false;
            for exist in anchor_existence[(block_index-quot)..block_index].iter().rev() {
                if *exist {
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
            }
            EmpBlock::new(
                odd_block_count*emp_kmer.odd + even_block_count*emp_kmer.even,
                block_len + odd_block_count + even_block_count + rem
            )
        };
        // hind block
        let hind_emp_block = {
            let ref_block_len = ref_len - (self.position.0 + kmer);
            let qry_block_len = qry_len - (self.position.1 + kmer);
            let block_len = min(ref_block_len, qry_block_len);
            let quot = block_len / kmer;
            let rem = block_len % kmer;
            let mut odd_block_count: usize = 0;
            let mut even_block_count: usize = 0;
            let mut previous_block_is_odd = false;
            for exist in anchor_existence[(block_index+1)..(block_index+quot+1)].iter() {
                if *exist {
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
            }
            EmpBlock::new(
                odd_block_count*emp_kmer.odd + even_block_count*emp_kmer.even,
                block_len + odd_block_count + even_block_count + rem
            )
        };
        self.state = AnchorState::Raw((fore_emp_block, hind_emp_block));
    }
}

enum AnchorState {
    Proto,
    Raw((EmpBlock, EmpBlock)), // Fore, Hind
    OnesideDone((AlignmentBlock, BlockType)),
    Dropped,
    Valid(AlignmentBlock),
}

struct AlignmentBlock {
    operations: Vec<Operation>,
    penalty: usize,
}

struct EmpBlock {
    penalty: usize,
    length: usize,
}

impl EmpBlock {
    fn new(penalty: usize, length: usize) -> Self {
        Self {
            penalty,
            length,
        }
    }
}

enum BlockType {
    Fore,
    Hind,
}