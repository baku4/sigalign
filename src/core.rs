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


pub type Sequence<'a> = &'a [u8];

pub trait Reference {
    fn locate(&self, pattern: Sequence, kmer: usize) -> Vec<RecordLocation>;
    fn sequence_of_record(&self, record_index: usize) -> Sequence;
    fn length_of_record(&self, record_index: usize) -> usize;
}

pub struct RecordLocation {
    pub index: usize,
    pub positions: Vec<usize>,
}


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

#[derive(Debug)]
struct AlignmentOperation {
    alignment_type: AlignmentType,
    count: u32,
}

#[derive(Debug)]
enum AlignmentType {
    Match,
    MisMatch,
    Insertion,
    Deletion,
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
struct Extension {
    penalty: usize,
    length: usize,
    operations: OperationsOfExtension,
}

#[derive(Debug)]
enum OperationsOfExtension {
    Own(OwnedOperations),
    Ref(PointerToOperations),
}

#[derive(Debug)]
struct OwnedOperations {
    operations: Vec<AlignmentOperation>,
}

#[derive(Debug)]
struct PointerToOperations {
    anchor_index: usize,
    start_point_of_operations: StartPointOfOperations,
}

#[derive(Debug)]
struct StartPointOfOperations {
    operation_index: usize,
    operation_count: usize,
}

#[derive(Debug)]
pub struct CheckPoints(Vec<usize>);


// ALGORITHM


trait Algorithm {
    fn semi_global_alignment(
        reference: &dyn Reference,
        query: Sequence,
        pattern_size: usize,
        cutoff: &Cutoff,
        penalties: &Penalties,
        min_penalty_for_pattern: &MinPenaltyForPattern,
    ) {
        let anchors_preset_by_record = Anchors::create_preset_by_record(reference, query, pattern_size);

        for (record_index, anchors_preset) in anchors_preset_by_record {
            let record_sequence = reference.sequence_of_record(record_index);
            let record_length = record_sequence.len();

            let mut anchors = Anchors::from_preset(anchors_preset, record_length, query, pattern_size, cutoff, penalties, min_penalty_for_pattern);

            anchors.extend(record_sequence, query);
        }
    }
}
