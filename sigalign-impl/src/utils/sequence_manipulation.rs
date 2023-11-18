const A_ASCII: u8 = 65;
const C_ASCII: u8 = 67;
const G_ASCII: u8 = 71;
const T_ASCII: u8 = 84;
const U_ASCII: u8 = 85;

const DNA_COMP_MAP: [u8; 128] = [
    0u8, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31,
    32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63,
    64, T_ASCII, 66, G_ASCII, 68, 69, 70, C_ASCII, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, A_ASCII, 85, 86, 87, 88, 89, 90, 91, 92, 93, 94, 95,
    96, 97, 98, 99, 100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115, 116, 117, 118, 119, 120, 121, 122, 123, 124, 125, 126, 127,
];
const RNA_COMP_MAP: [u8; 128] = [
    0u8, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31,
    32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63,
    64, U_ASCII, 66, G_ASCII, 68, 69, 70, C_ASCII, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, A_ASCII, 86, 87, 88, 89, 90, 91, 92, 93, 94, 95,
    96, 97, 98, 99, 100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115, 116, 117, 118, 119, 120, 121, 122, 123, 124, 125, 126, 127,
];

/// Convert the A to T, C to G, G to C and T to A with reversed.
#[inline]
pub fn reverse_complement_of_dna(sequence: &[u8]) -> Vec<u8> {
    sequence.iter().rev().map(|&character| DNA_COMP_MAP[character as usize]).collect()
}
#[inline]
pub fn reverse_complement_of_rna(sequence: &[u8]) -> Vec<u8> {
    sequence.iter().rev().map(|&character| RNA_COMP_MAP[character as usize]).collect()
}
#[inline]
pub fn get_unique_characters_of_sequence(sequence: &[u8]) -> Vec<u8> {
    let mut table = [false; 256];
    for chr in sequence {
        table[*chr as usize] = true;
    }
    table.iter().enumerate().filter_map(|(idx, c)| {
        if *c { Some(idx as u8) } else { None }
    }).collect()
}
