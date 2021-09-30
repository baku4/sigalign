mod anchoring;
mod extending;

// CONDITIONS

pub struct Penalties {
    pub x: usize,
    pub o: usize,
    pub e: usize,
}

pub struct Cutoff {
    pub minimum_aligned_length: usize,
    pub penalty_per_length: f32,
}

pub struct MinPenaltyForPattern {
    pub odd: usize,
    pub even: usize,
}

// TEXT

pub type Query<'a> = &'a [u8];

pub trait Reference {
    fn locate(&self, pattern: Query, kmer: usize) -> Vec<RecordLocation>;
    fn length_of_record(&self, record_index: usize) -> usize;
}

pub struct RecordLocation {
    pub index: usize,
    pub positions: Vec<usize>,
}


// ANCHOR

#[derive(Debug)]
struct Anchors {
    anchors: Vec<Anchor>,
}

#[derive(Debug)]
struct Anchor {
    query_position: usize,
    record_position: usize,
    size: usize,
    left_estimation: Estimation,
    right_estimation: Estimation,
    left_check_points: CheckPoints,
    right_check_points: CheckPoints,
    left_extension: Option<Extension>,
    right_extension: Option<Extension>,
    dropped: bool,
}

#[derive(Debug)]
struct Estimation {
    penalty: usize,
    length: usize,
}

#[derive(Debug)]
enum Extension {
    Own,
    Ref,
}

#[derive(Debug)]
pub struct CheckPoints(Vec<usize>);

// RESULTS

pub struct AlignmentResult {
    dissimilarity: f32,
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


trait Algorithm {
    fn semi_global_alignment(
        reference: &dyn Reference,
        query: Query,
        pattern_size: usize,
        cutoff: &Cutoff,
        penalties: &Penalties,
        min_penalty_for_pattern: &MinPenaltyForPattern,
    ) {
        let mut anchors_by_record  = Anchors::new_for_semi_global(reference, query, pattern_size, cutoff, penalties, min_penalty_for_pattern);


    }
}
