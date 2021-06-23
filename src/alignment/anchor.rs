use super::{FmIndex, Operation, AlignmentBlock};
use fm_index::BackwardSearchIndex;

struct AnchorGroup<'a, 'b> {
    ref_seq: &'a [u8],
    qry_seq: &'a [u8],
    kmer: usize,
    anchors: Vec<Anchor<'b>>,
}
impl<'a, 'b> AnchorGroup<'a, 'b> {
    fn new(ref_seq: &[u8], qry_seq: &[u8], index: &FmIndex, kmer: usize) { // -> Self
        let ref_len = ref_seq.len();
        let qry_len = qry_seq.len();
        let anchor_count = qry_len / kmer;
        let mut positions = Vec::with_capacity(anchor_count);
        for i in 0..anchor_count {
            let qry_position = i*kmer;
            let pattern = &qry_seq[qry_position..qry_position+kmer];
            let search = index.search_backward(pattern);
            positions.push(search.locate());
        }
        // block_existence_info
        let block_existence_info: Vec<bool> = positions.iter().map(|x| x.len() != 0).collect();
        // generate anchors
        for (idx, position) in positions.iter().enumerate() {
            
            // unpeccable extension
        }
    }
    fn is_empty() {

    }
}

struct Anchor<'a> {
    state: AnchorState,
    check_points: Vec<&'a Self>,
}

impl<'a> Anchor<'a> {
    fn new(ref_len: usize, qry_len: usize, ref_pos: usize, qry_pos: usize, kmer: usize) {

    }
}

enum AnchorState {
    Raw((EmpBlock, EmpBlock)), // Fore, Hind
    OnesideDone((AlignmentBlock, BlockType)),
    Dropped,
    Valid(AlignmentBlock),
}
struct EmpBlock {
    penalty: usize,
    length: usize,
}
impl EmpBlock {
    fn new(ref_len: usize, block_existence_info: Vec<bool>, qry_len: usize, kmer:usize) {

    }
}


enum BlockType {
    Fore,
    Hind,
}