pub struct AlignmentResult {
    penalty: usize,
    length: usize,
    position: AlignmentPosition,
    operations: Vec<AlignmentOperation>,
}

struct AlignmentPosition {
    reference: (usize, usize),
    query: (usize, usize),
}

struct AlignmentOperation {
    alignment_type: AlignmentType,
    count: u32,
}

enum AlignmentType {
    Match,
    MisMatch,
    Insertion,
    Deletion,
}
