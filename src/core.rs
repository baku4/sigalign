mod aligner;

pub struct Penalties {
    x: usize,
    o: usize,
    e: usize,
}

pub struct Cutoff {
    minimum_aligned_length: usize,
    penalty_per_length: f32,
}

pub trait Aligner {
    fn new(penalties: Penalties, cutoff: Cutoff) -> Self;
    fn align(reference: &dyn Reference, search_range: &SearchRange, query: Query) -> AlignmentResult;
}

trait Reference {
    
}

struct SearchRange {
    sorted_vector: Vec<usize>,
}

type Query<'a> = &'a [u8];

struct AlignmentResult {
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
