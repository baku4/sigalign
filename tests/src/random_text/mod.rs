use std::ops::Range;

use rand::Rng;

pub const UTF8_OF_NO: [u8; 4] = [b'A', b'C', b'G', b'T'];
pub const UTF8_OF_NN: [u8; 5] = [b'A', b'C', b'G', b'T', b'_'];
pub const UTF8_OF_AO: [u8; 20] = [b'A', b'C', b'D', b'E', b'F', b'G', b'H', b'I', b'K', b'L', b'M', b'N', b'P', b'Q', b'R', b'S', b'T', b'V', b'W', b'Y'];
pub const UTF8_OF_AN: [u8; 21] = [b'A', b'C', b'D', b'E', b'F', b'G', b'H', b'I', b'K', b'L', b'M', b'N', b'P', b'Q', b'R', b'S', b'T', b'V', b'W', b'Y', b'_'];

const TEXT_LENGTH_RANGE: Range<usize> = 50..200;

pub fn rand_text_of_no() -> Vec<u8> {
    rand_text_of_list(&UTF8_OF_NO)
}
pub fn rand_text_of_nn() -> Vec<u8> {
    rand_text_of_list(&UTF8_OF_NN)
}
pub fn rand_text_of_ao() -> Vec<u8> {
    rand_text_of_list(&UTF8_OF_AO)
}
pub fn rand_text_of_an() -> Vec<u8> {
    rand_text_of_list(&UTF8_OF_AN)
}
pub fn rand_pattern_of_text(text: &[u8]) -> Vec<u8> {
    let mut rng = rand::thread_rng();
    let start = rng.gen_range(0..text.len() - 1);
    let end = rng.gen_range(start+1..text.len());
    text[start..end].to_vec()
}

fn rand_text_of_list(chr_list: &[u8]) -> Vec<u8> {
    let mut rng = rand::thread_rng();
    let text_len: usize = rng.gen_range(TEXT_LENGTH_RANGE);
    rand_text_with_length(chr_list, text_len)
}
pub fn rand_text_with_length(chr_list: &[u8], text_len: usize) -> Vec<u8> {
    let mut rng = rand::thread_rng();
    let list_length = chr_list.len();
    let text: Vec<u8> = (0..text_len).map(|_| chr_list[rng.gen_range(0..list_length)]).collect();
    text
}