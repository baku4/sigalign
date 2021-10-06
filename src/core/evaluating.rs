use super::Cutoff;
use super::Extension;
use super::Anchors;

use std::collections::HashSet;

type AnchorSymbol = Vec<usize>;

impl Anchors {
    fn get_anchors_symbol(
        &self,
        cutoff: &Cutoff,
    ) {
        let valid_anchors = self.set_of_valid_anchors(cutoff);
        let mut used_symbols: HashSet<AnchorSymbol> = HashSet::with_capacity(valid_anchors.len());
        let mut unique_anchors: Vec<usize> = Vec::with_capacity(valid_anchors.len());

        for &valid_anchor_index in &valid_anchors {
            let symbol = self.get_symbol_of_anchor(valid_anchor_index);
            let symbol_is_new = used_symbols.insert(symbol);
            if symbol_is_new {
                unique_anchors.push(valid_anchor_index);
            }
        };
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
    fn get_symbol_of_anchor(&self, anchor_index: usize) -> AnchorSymbol {
        let mut checked_anchors: HashSet<usize> = HashSet::new();

        let mut symbol: AnchorSymbol = Vec::new();
        
        let mut to_check_anchors: HashSet<usize> = HashSet::with_capacity(1);
        to_check_anchors.insert(anchor_index);
        
        while to_check_anchors.len() != 0 {
            to_check_anchors.iter().map(|&to_check_anchor_index| {
                for connected_anchor_index in &self.anchors[to_check_anchor_index].connected_anchors {
                    
                }
            });
            ()
        }
        
        symbol
    }
}