use wasm_bindgen::prelude::*;
use rand::prelude::*;

const MIN_STEM_LEN: usize = 300;
const MAX_STEM_LEN: usize = 400;
const MIN_EDGE_LEN: usize = 0;
const MAX_EDGE_LEN: usize = 100;

/*
-----------------------------
| 0~100   |    300~400      |  0~100
          ----------------------------

Two pairs of sequences that look like above + one random one
-> total 5 target sequences (range in 300..=500)
*/
#[wasm_bindgen]
pub fn get_sample_target_as_fasta_string() -> String {
    let mut record_sequences: Vec<String> = Vec::with_capacity(5);

    let mut rng = rand::thread_rng();
    for _ in 0..2 {
        let stem_len = rng.gen_range(MIN_STEM_LEN..=MAX_STEM_LEN);
        let stem = get_random_seq(&mut rng, stem_len);

        let left_edge_len = rng.gen_range(MIN_EDGE_LEN..=MAX_EDGE_LEN);
        let left_edge = get_random_seq(&mut rng, left_edge_len);

        let right_edge_len = rng.gen_range(MIN_EDGE_LEN..=MAX_EDGE_LEN);
        let right_edge = get_random_seq(&mut rng, right_edge_len);

        record_sequences.push(left_edge + &stem);
        record_sequences.push(stem + &right_edge);
    }
    let seq_len = rng.gen_range(MIN_STEM_LEN+MIN_EDGE_LEN..=MAX_STEM_LEN+MAX_EDGE_LEN);
    let seq = get_random_seq(&mut rng, seq_len);
    record_sequences.push(seq);
    
    record_sequences.shuffle(&mut rng);

    format!(
        ">first_record\n{}\n>second_record\n{}\n>third_record\n{}\n>fourth_record\n{}\n>fifth_record\n{}",
        record_sequences[0], record_sequences[1], record_sequences[2], record_sequences[3], record_sequences[4]
    )
}

#[inline]
fn get_random_seq(rng: &mut ThreadRng, seq_len: usize) -> String {
    let seq = (0..seq_len).map(|_| get_random_chr(rng)).collect::<Vec<u8>>();
    String::from_utf8(seq).unwrap()
}

const NUCLEOTIDES: [u8; 4] = [b'A', b'C', b'G', b'T'];
#[inline]
fn get_random_chr(rng: &mut ThreadRng) -> u8 {
    NUCLEOTIDES[rng.gen_range(0..NUCLEOTIDES.len())]
}
