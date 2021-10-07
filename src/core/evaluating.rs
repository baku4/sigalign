use super::Cutoff;
use super::{AlignmentResult};
use super::Extension;
use super::{Anchors, Anchor};

use std::collections::HashSet;

type AnchorSymbol = Vec<usize>;

impl Anchors {
    fn get_alignment_result(
        self,
        cutoff: &Cutoff,
    ) {
        let unique_anchors = self.get_unique_anchors(cutoff);

        for unique_anchor_index in unique_anchors {
            unique_anchor_index
        }
    }
    fn get_alignment_result_of_anchor(
        &self,
        anchor_index: usize,
    ) {
        let anchor = &self.anchors[anchor_index];

        let left_extension = &anchor.left_extension.unwrap();
        let right_extension = &anchor.right_extension.unwrap();
        
        let penalty = left_extension.penalty + right_extension.penalty;
        let length = left_extension.length + anchor.size + right_extension.length;

        AlignmentResult
    }
    fn get_unique_anchors(
        &self,
        cutoff: &Cutoff,
    ) -> Vec<usize> {
        let valid_anchors = self.set_of_valid_anchors(cutoff);

        let mut used_symbols: HashSet<AnchorSymbol> = HashSet::with_capacity(valid_anchors.len());
        let mut unique_anchors: Vec<usize> = Vec::with_capacity(valid_anchors.len());

        for valid_anchor_index in valid_anchors {
            let symbol = self.anchors[valid_anchor_index].get_symbol(valid_anchor_index);
            let symbol_is_new = used_symbols.insert(symbol);
            if symbol_is_new {
                unique_anchors.push(valid_anchor_index);
            }
        };

        unique_anchors
    }
    fn set_of_valid_anchors(
        &self,
        cutoff: &Cutoff,
    ) -> HashSet<usize> {
        self.anchors.iter().enumerate().filter_map(|(anchor_index, anchor)| {
            if !anchor.dropped && {
                let left_extension = anchor.left_extension.as_ref().unwrap();
                let right_extension = anchor.right_extension.as_ref().unwrap();
                let penalty = left_extension.penalty + right_extension.penalty;
                let length = left_extension.length + anchor.size + right_extension.length;
                
                length >= cutoff.minimum_aligned_length && (penalty as f32 / length as f32) <= cutoff.penalty_per_length
            } {
                Some(anchor_index)
            } else {
                None
            }
        }).collect()
    }
}

impl Anchor {
    fn get_symbol(&self, anchor_index: usize) -> AnchorSymbol {
        let mut symbol = self.connected_anchors.clone();
        symbol.push(anchor_index);
        symbol.sort();
        symbol
    }
}