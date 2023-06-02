/*!
Retest the pattern index
*/
use sigalign::{
    reference::{
        pattern_index::{
            PatternIndex,
            PatternIndexBuildError,
            ConcatenatedSequenceWithBoundaries,
            lfi::{
                Lfi32B2V64, LfiOption,
            },
        },
    },
};
use crate::random_text_and_pattern::{
    gen_rand_chr_list,
    gen_rand_text,
};
use ahash::{AHashMap, AHashSet};

#[test]
fn accurate_pattern_location_in_pattern_index() {
    // Configs
    let chr_count = 4;
    let chr_list = gen_rand_chr_list(chr_count);
    let text_lengths = [10, 20, 30, 40, 50];
    let pattern_len = 5;
    
    // Generate texts
    let texts: Vec<Vec<u8>> = text_lengths.iter().map(|text_len| {
        gen_rand_text(&chr_list, *text_len, *text_len)
    }).collect();

    // Concatenate text
    let concated_text: Vec<u8> = texts.iter().flatten().map(|x| *x).collect();

    // Get patterns
    let patterns: Vec<Vec<u8>> = concated_text
        .windows(pattern_len)
        .map(|x| x.to_vec())
        .collect();

    // Get answers
    let answers = {
        let mut answers = AHashMap::new();

        for (text_idx, text) in texts.iter().enumerate() {
            text.windows(pattern_len).enumerate()
                .for_each(|(pattern_idx, pattern)| {
                    let string = String::from_utf8(pattern.to_vec()).unwrap();

                    if !answers.contains_key(&string) {
                        answers.insert(string.clone(), Vec::new());
                    };
                    answers.get_mut(&string).unwrap().push((text_idx, pattern_idx));
                });
        }
        answers
    };

    // concatenated_sequence_with_boundaries
    let mut boundaries = vec![0];
    let mut accumed_len = 0;
    for v in text_lengths {
        accumed_len += v;
        boundaries.push(accumed_len as u64);
    };
    let concatenated_sequence_with_boundaries = ConcatenatedSequenceWithBoundaries {
        concatenated_sequence: concated_text.clone(),
        boundaries,
    };

    let alignable_sequence: Vec<u8> = {
        let set: AHashSet<u8> = concated_text.iter().cloned().collect();
        set.iter().cloned().collect()
    };
    // Pattern index
    let pattern_index = Lfi32B2V64::new(
        &alignable_sequence,
        concatenated_sequence_with_boundaries,
        LfiOption { suffix_array_sampling_ratio: 1, lookup_table_kmer_size: 2 },
    ).unwrap();
    let search_range: Vec<u32> = (0..text_lengths.len()).map(|x| x as u32).collect();
    
    for pattern in patterns {
        let pattern_locations = pattern_index.locate(&pattern, &search_range);
        for pattern_location in pattern_locations {
            let answser = answers.get(&String::from_utf8(pattern.to_vec()).unwrap()).unwrap();

            let target_index = pattern_location.target_index;
            let sorted_positions = pattern_location.sorted_positions;
            for position in sorted_positions {
                assert!(answser.contains(&(target_index as usize, position as usize)));
            }
        }
    }
}