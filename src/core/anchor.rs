mod preset;

pub use preset::AnchorPreset;

struct AnchorsFromPattern {
    qry_pos: usize,
    anchors: Vec<Anchor>, // sorted by reference positions
}

struct Anchor {
    ref_pos: usize,
    size: usize,
    left_alignment: AlignmentBlock,
    right_alignment: AlignmentBlock,
    left_checkpoint: CheckPoint,
    right_checkpoint: CheckPoint,
    dropped: bool,
}

struct AlignmentBlock {

}

struct CheckPoint(Vec<usize>); // Index of pre "anchors from pattern"
