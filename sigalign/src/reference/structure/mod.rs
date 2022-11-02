use super::{
    Result, error_msg,
    Sequence,
    PatternLocation,
};
use super::{
    // Requirement for struct
    Serializable, SizeAware,
};

mod pattern_finder;
mod sequence_type;

pub use pattern_finder::{PatternFinder, JoinedSequence};
pub use sequence_type::SequenceType;


#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    #[test]
    fn save_and_load_sequence_type_and_pattern_finder() {
        let sequence_type_1 = SequenceType::new_nucleotide_only();
        let sequence_type_2 = SequenceType::new_nucleotide_with_noise(100);
        let sequence_type_3 = SequenceType::new_amino_acid_only();
        let sequence_type_4 = SequenceType::new_amino_acid_with_noise(100);

        let pattern_finder_1 = pattern_finder_for_test(200);
        let pattern_finder_2 = pattern_finder_for_test(500);
        let pattern_finder_3 = pattern_finder_for_test(1000);
        let pattern_finder_4 = pattern_finder_for_test(2000);
        
        for (sequence_type, pattern_finder) in [
            (sequence_type_1, pattern_finder_1),
            (sequence_type_2, pattern_finder_2),
            (sequence_type_3, pattern_finder_3),
            (sequence_type_4, pattern_finder_4),
        ] {
            let mut buffer = Vec::new();

            pattern_finder.save_to(&mut buffer).unwrap();
            sequence_type.save_to(&mut buffer).unwrap();

            let mut cursor = Cursor::new(buffer);

            let pattern_finder_loaded = PatternFinder::load_from(&mut cursor).unwrap();
            let sequence_type_loaded = SequenceType::load_from(&mut cursor).unwrap();

            assert_eq!(sequence_type, sequence_type_loaded);
            assert_eq!(pattern_finder, pattern_finder_loaded);
        }
    }

    fn pattern_finder_for_test(text_len: usize) -> PatternFinder {
        let nc = b"ACGT";

        let text: Vec<u8> = (0..text_len).map(|x| {
            let idx = x % 4;
            nc[idx]
        }).collect();

        let position = vec![
            0,
            (text_len % 4) as u64,
            (text_len % 3) as u64,
            (text_len % 2) as u64,
            text_len as u64,
        ];

        let joined_sequence = JoinedSequence::new(text, position);

        PatternFinder::new(
            joined_sequence,
            true,
            false,
            true,
            2,
            4,
        ).unwrap()
    }
}