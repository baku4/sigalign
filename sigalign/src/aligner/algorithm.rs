// Alignment algorithms
use super::{
	Penalties, PRECISION_SCALE, Cutoff, MinPenaltyForPattern,
	AlignmentResult, RecordAlignmentResult, AnchorAlignmentResult, AlignmentPosition, AlignmentOperation, AlignmentCase,
    Sequence,
    ReferenceInterface, SequenceBuffer, PatternLocation,
    Reference, SequenceProvider,
};

mod common_steps_dep;
use common_steps_dep::{AlignmentHashSet};

mod semi_global_dep;
mod local_dep;

pub use local_dep::local_alignment_algorithm;
pub use semi_global_dep::semi_global_alignment_algorithm;

// New version!
// Common steps
mod pos_table;
use pos_table::{PosTable, AnchorPosition, AnchorIndex};
mod spare_penalty;
use spare_penalty::{calculate_spare_penalty};
mod extending;
use extending::{Extension, WaveEndPoint, WaveFrontScore, Components, Component, BackTraceMarker};
pub use extending::WaveFront;
mod backtrace;
use backtrace::{TraversedPosition, TraversedAnchors, TraversedAnchor};
mod merging;

mod semi_global;

#[cfg(test)]
mod tests {
    use crate::{sequence_provider::*, ReferenceBuilder};
    use super::*;

    #[test]
    fn print_wave_front_backtrace_traversed() {
        let record_seq = b"AGCGTTTTATTACCTTTTGAATCCCAAAACATACATGCAGCATTCATTTTGCCACCAGTTTTTTTCATGCTTGATTCATATATAGCCTTTCTATCAGGAGATACTGTTTCTCCATGCTGCATACACAATTTTCGATAAGCATCATCATCCCTTTTTCCAGTAGCAAACTCTTTTCTTGCAAGTTCTTTTATTGCTTCGTCAAATTCTTCCTCTGACATCGCTGGTTTATCTCGTTTTGTCATGATAGTATCCCAGTTTGGTTTGGTAAAATTAATGTCCACAGGCTTAAATCTTAATGAG";
        let query_seq_1 = b"AGCGTTTTATTACCTTTTGAATCCCAAAACATACATGCAGCATTCATTTTGCCACCGTTTTTTTCATGCTTGATTCATATATAGCCTTTCTATCAGGAGCTACTGTTTCTCCATGCTGCATACACAATTTTCGATAAGCCATCATCATCCCTTTTTCCAGTAGCAAACTCTTTTCTTGCAAGTTCTTTTATTGCTTCGTCAAATTCTTCCTCTGACATCGCTGGTTTATCTCGTTTTGTCATGATAGTATCCCAGTTTGGTTTGGTAAAATTAATGTCCACAGGCTTAAATCTTAATGAG";
        let query_seq_2 = b"AGCGTTTTATTACCTTTTGAATCCCAAAACATACATGCAACATTCATTTTGCCACCAGTTATTTTCATGCTTGATTCATATATAGCCTTTCTATCAGGAATACTGTTTCTCCATGCTGCAACACAATTTTCGATAAGCATCATCATCCCTTTTTCCAGTAGCAAACTCTTTTCTTGCAAGTTCTTTTATTGCTTCGTCAAATTCTTCCTCTGACATCGGGTTTATCTGTTTTGTCATGATAGTATCCCAGTTTGGTTTGGATAAAATTAATGTCCACAGGCTTAAATCTTAATGAG";

        let pattern_size = 20;

        // Pos Table
        let mut sequence_provider = InMemoryProvider::new();
        sequence_provider.add_record(record_seq, "record_1");
        let reference = ReferenceBuilder::new().build(sequence_provider).unwrap();
        let pos_table = PosTable::new_by_record(&reference, query_seq_1, pattern_size).get(&0).unwrap().clone();

        println!("pos_table: {:#?}", pos_table);

        // WaveFront
        let penalties = Penalties {
            x: 5,
            o: 6,
            e: 3,
        };
        let mut wave_front = WaveFront::new_allocated(&penalties, 100);

        // Right
        println!("# Right");
        {
            let anchor_pattern_index = 0;
        
            let anchor_position = &pos_table.0[anchor_pattern_index][0];
            let anchor_size = anchor_position.pattern_count * pattern_size;
            let record_start_position = anchor_position.record_position + anchor_size;
            let query_start_position = anchor_pattern_index * pattern_size + anchor_size;
            let record_slice = &record_seq[record_start_position..];
            let query_slice = &query_seq_1[query_start_position..];

            wave_front.align_right_to_end_point(record_slice, query_slice, &penalties, 100);
            println!("end_point: {:#?}", &wave_front.end_point);
            let end_score = wave_front.end_point.score;
            let end_k = wave_front.wave_front_scores[wave_front.end_point.score].max_k + wave_front.end_point.k.unwrap();
            let (mut extension, traversed_positions) = wave_front.backtrace_from_point_checking_right_traversed(end_score, end_k as usize, &penalties, pattern_size);

            println!("extension: {:#?}", extension);
            println!("traversed_positions: {:?}", traversed_positions);

            let traversed_anchors = pos_table.right_traversed_anchors(
                traversed_positions,
                anchor_pattern_index,
                anchor_position.pattern_count,
                record_start_position,
                extension.length,
                extension.penalty,
                pattern_size,
            );
            println!("traversed_anchors: {:?}", traversed_anchors);
        }

        // Left
        println!("# Left");
        {
            let anchor_pattern_index = 7;
        
            let anchor_position = &pos_table.0[anchor_pattern_index][0];
            let anchor_size = anchor_position.pattern_count * pattern_size;
            let record_last_position = anchor_position.record_position;
            let query_last_position = anchor_pattern_index * pattern_size;
            let record_slice = &record_seq[..record_last_position];
            let query_slice = &query_seq_1[..query_last_position];

            wave_front.align_left_to_end_point(record_slice, query_slice, &penalties, 100);
            println!("end_point: {:#?}", &wave_front.end_point);
            let end_score = wave_front.end_point.score;
            let end_k = wave_front.wave_front_scores[wave_front.end_point.score].max_k + wave_front.end_point.k.unwrap();
            let (mut extension, traversed_positions) = wave_front.backtrace_from_point_checking_left_traversed(end_score, end_k as usize, &penalties, pattern_size);

            println!("extension: {:#?}", extension);
            println!("traversed_positions: {:?}", traversed_positions);

            let traversed_anchors = pos_table.left_traversed_anchors(traversed_positions, anchor_pattern_index, record_last_position, extension.length, extension.penalty, pattern_size);
            println!("traversed_anchors: {:?}", traversed_anchors);
        }
    }
}