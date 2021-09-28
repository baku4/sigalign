use std::collections::HashMap;

mod anchor;

mod semi_global;
mod local;

type Query<'a> = &'a [u8];

struct Algorithm();

impl Algorithm {
    fn create_anchors_by_record(
        reference: &dyn Reference,
        query: Query,
        kmer: usize,
    ) {
        let qry_len = query.len();
        let pattern_count = qry_len / kmer;

        let mut anchor_preset = anchor::AnchorPreset::new();

        for pattern_idx in 0..pattern_count {
            let qry_pos = pattern_idx * kmer;
            let pattern = &query[qry_pos..qry_pos+kmer];

            let sorted_ref_positions = reference.locate_positions(pattern, kmer);

            anchor_preset.add_positions_of_pattern(pattern_idx, sorted_ref_positions);
        }

        // let anchors = anchor_preset.to_achors();
    }
}

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

trait Reference {
    fn locate_positions(&self, pattern: Query, kmer: usize) -> ReferencePositions;
}

struct ReferencePositions(HashMap<usize, Vec<usize>>); //TODO: vector can be used

struct SearchRange {
    sorted_vector: Vec<usize>,
}
