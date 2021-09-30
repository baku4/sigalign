use std::{collections::HashMap, hash::Hash};

mod anchor;
mod extension;

use anchor::{AnchorsPreset, Anchors};

type Query<'a> = &'a [u8];

trait Algorithm {
    fn create_anchors_for_semi_global(
        reference: &dyn Reference,
        query: Query,
        kmer: usize,
        min_penalty_for_pattern: &MinPenaltyForPattern,
    ) {
        let anchors_preset_by_record = Self::create_anchors_preset_by_record(reference, query, kmer);
        let anchors_by_record: HashMap<usize, Anchors> = anchors_preset_by_record.into_iter().map(|(record_index, anchors_preset)| {
            let record_length = reference.length_of_record(record_index);
            
            let anchors = anchors_preset.to_anchors_for_semi_global(
                kmer,
                query.len(),
                record_length,
                min_penalty_for_pattern,
            );

            (record_index, anchors)
        }).collect();
    }
    fn create_anchors_preset_by_record(
        reference: &dyn Reference,
        query: Query,
        kmer: usize,
    ) -> HashMap<usize, AnchorsPreset> {
        let qry_len = query.len();
        let pattern_count = qry_len / kmer;

        let mut anchors_preset_by_record: HashMap<usize, AnchorsPreset> = HashMap::new();

        for pattern_index in 0..pattern_count {
            let qry_pos = pattern_index * kmer;
            let pattern = &query[qry_pos..qry_pos+kmer];

            let reference_location = reference.locate(pattern, kmer);

            for record_location in reference_location {
                match anchors_preset_by_record.get_mut(&record_location.index) {
                    Some(anchors_preset) => {
                        anchors_preset.add_new_position(pattern_index, record_location.positions)
                    },
                    None => {
                        let mut new_anchors_preset = AnchorsPreset::new(pattern_count);
                        new_anchors_preset.add_new_position(pattern_index, record_location.positions);
                        anchors_preset_by_record.insert(record_location.index, new_anchors_preset);
                    }
                }
            }
        }

        anchors_preset_by_record
    }
}

trait Reference {
    fn locate(&self, pattern: Query, kmer: usize) -> Vec<RecordLocation>;
    fn length_of_record(&self, record_index: usize) -> usize;
}

struct RecordLocation {
    index: usize,
    positions: Vec<usize>,
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
