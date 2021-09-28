pub struct AnchorsPreset {
    total_pattern_count: usize,
    anchor_positions: Vec<AnchorPosition>,
}

impl AnchorsPreset {
    pub fn new(pattern_count: usize) -> Self {
        Self {
            total_pattern_count: pattern_count,
            anchor_positions: Vec::new(),
        }
    }
    pub fn add_new_position(&mut self, pattern_index: usize, record_positions: Vec<usize>) {
        let new_anchor_position = AnchorPosition::new(pattern_index, record_positions);
        self.anchor_positions.push(new_anchor_position);
    }
    pub fn to_anchors(&self, record_length: usize, pattern_size: usize) {
        let matched_pattern_count = self.anchor_positions.len();
        let mut left_unmatched_pattern_count = 0;
        let mut right_unmatched_pattern_count = self.total_pattern_count - matched_pattern_count;

        for anchor_position in &self.anchor_positions {

            let pattern_index = anchor_position.pattern_index;
            let record_positions = &anchor_position.record_positions;
            
            let left_query_length = left_unmatched_pattern_count * pattern_size;
            let right_query_length = right_unmatched_pattern_count * pattern_size;
        }
    }
}

struct AnchorPosition {
    pattern_index: usize,
    record_positions: Vec<usize>,
}

impl AnchorPosition {
    fn new(pattern_index: usize, record_positions: Vec<usize>) -> Self {
        Self {
            pattern_index,
            record_positions,
        }
    }
}

//
//

struct Anchor {
    ref_pos: usize,
    size: usize,
    left_extension: Extension,
    right_extension: Extension,
    left_checkpoints: CheckPoints,
    right_checkpoints: CheckPoints,
    dropped: bool,
}

enum Extension {
    Estimated(),
    Exact(),
}

struct CheckPoints {
    query_position: usize,

}
