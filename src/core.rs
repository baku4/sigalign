mod anchoring;
mod extending;
mod evaluating;


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

#[derive(Debug, Clone)]
struct AlignmentOperation {
    alignment_type: AlignmentType,
    count: u32,
}

#[derive(Debug, Clone)]
enum AlignmentType {
    Match,
    Subst,
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
    left_checkpoints: CheckPoints,
    right_checkpoints: CheckPoints,
    left_extension: Option<Extension>,
    right_extension: Option<Extension>,
    dropped: bool,
    connected_anchors: Vec<usize>,
}

#[derive(Debug)]
struct Estimation {
    penalty: usize,
    length: usize,
}

#[derive(Debug, Clone)]
pub struct Extension {
    penalty: usize,
    length: usize,
    operations: OperationsOfExtension,
}

#[derive(Debug, Clone)]
enum OperationsOfExtension {
    Own(OwnedOperations),
    Ref(RefToOperations),
}

#[derive(Debug, Clone)]
struct OwnedOperations {
    operations: Vec<AlignmentOperation>,
}

#[derive(Debug, Clone)]
struct RefToOperations {
    anchor_index: usize,
    start_point_of_operations: StartPointOfOperations,
}

#[derive(Debug, Clone)]
struct StartPointOfOperations {
    operation_index: usize,
    operation_count: u32,
}

#[derive(Debug)]
pub struct CheckPoints(Vec<CheckPoint>);

#[derive(Debug)]
pub struct CheckPoint {
    anchor_index: usize,
    anchor_size: u32,
    record_position_gap: u32,
    query_position_gap: u32,
}


// ALGORITHM


trait Algorithm {
    fn semi_global_alignment(
        reference: &dyn Reference,
        query: Sequence,
        pattern_size: usize,
        penalties: &Penalties,
        cutoff: &Cutoff,
        min_penalty_for_pattern: &MinPenaltyForPattern,
    ) {
        let anchors_preset_by_record = Anchors::create_preset_by_record(reference, query, pattern_size);

        for (record_index, anchors_preset) in anchors_preset_by_record {
            let record_sequence = reference.sequence_of_record(record_index);
            let record_length = record_sequence.len();

            let mut anchors = Anchors::from_preset(anchors_preset, record_length, query, pattern_size, cutoff, penalties, min_penalty_for_pattern);

            println!("# record_index: {}", record_index);
            println!("# Anchoring:\n{:#?}", anchors);

            anchors.extend_for_semi_global(record_sequence, query, penalties, cutoff);
            
            println!("# Extending:\n{:#?}", anchors);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::reference::TestReference;

    struct TestAlgorithm;

    impl Algorithm for TestAlgorithm {}

    fn max_kmer_satisfying_cutoff(cutoff: &Cutoff, min_penalty_for_pattern: &MinPenaltyForPattern) -> usize {
        let mut n = 1;
        loop {
            let upper_bound = ((cutoff.minimum_aligned_length + 4)  as f32 / (2*n)  as f32 - 2_f32).ceil();
            let lower_bound = ((cutoff.minimum_aligned_length + 4)  as f32 / (2*n + 2)  as f32 - 2_f32).ceil();
            let max_penalty = (
                ((n*(min_penalty_for_pattern.odd + min_penalty_for_pattern.even)) as f32
                + 4_f32*cutoff.penalty_per_length) /
                (2_f32*cutoff.penalty_per_length*(n+1) as f32)
                - 2_f32
            ).ceil();

            let kmer = max_penalty.min(upper_bound);
            #[cfg(test)]
            println!("#n {}\nu {}\nl {}\nmp {}\nk {}", n, upper_bound, lower_bound, max_penalty, kmer);

            if kmer >= lower_bound {
                return kmer as usize
            }
            n += 1;
        }
    }

    #[test]
    fn test_semi_global() {
        let test_reference = TestReference::new();

        let query = b"CGGATGCTCCGGCAGCCGACAGAACGAAGGATCTTGCCGGAAAATGAACTTCTGTTATTATTTTTGTGATTCA";

        let penalties = Penalties {x: 4, o: 5, e: 2};
        let cutoff = Cutoff { minimum_aligned_length: 30, penalty_per_length: 0.3 };
        let min_penalty_for_pattern = MinPenaltyForPattern { odd: 4, even: 3 };

        let kmer = max_kmer_satisfying_cutoff(&cutoff, &min_penalty_for_pattern);

        let t = TestAlgorithm::semi_global_alignment(
            &test_reference,
            query,
            kmer,
            &penalties,
            &cutoff,
            &min_penalty_for_pattern
        );
    }
}