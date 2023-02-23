const A_ASCII: u8 = 65;
const C_ASCII: u8 = 67;
const G_ASCII: u8 = 71;
const T_ASCII: u8 = 84;

/// Convert the A to T, C to G, G to C and T to A with reversed.
pub fn reverse_complement_of_nucleotide(sequence: &[u8]) -> Vec<u8> {
    sequence.iter().rev().map(|&character| {
        match character{
            A_ASCII => T_ASCII,
            C_ASCII => G_ASCII,
            G_ASCII => C_ASCII,
            T_ASCII => A_ASCII,
            _ => character,
        }
    }).collect()
}