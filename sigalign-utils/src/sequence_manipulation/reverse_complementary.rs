/// Returns the reverse complement of a DNA sequence.
/// A, C, G, T are converted to T, G, C, A and other characters are remain unchanged.
pub fn reverse_complement_of_dna_sequence(sequence: &[u8]) -> Vec<u8> {
    let mut reverse_complement = Vec::with_capacity(sequence.len());
    for base in sequence.iter().rev() {
        reverse_complement.push(match base {
            b'A' => b'T',
            b'C' => b'G',
            b'G' => b'C',
            b'T' => b'A',
            _ => *base,
        });
    }
    reverse_complement
}

/// Returns the reverse complement of a DNA sequence (in-place).
pub fn reverse_complement_of_dna_sequence_in_place(sequence: &mut [u8]) {
    let mut i = 0;
    let mut j = sequence.len() - 1;
    while i < j {
        sequence.swap(i, j);
        i += 1;
        j -= 1;
    }
    sequence.iter_mut().for_each(|x| {
        *x = match x {
            b'A' => b'T',
            b'C' => b'G',
            b'G' => b'C',
            b'T' => b'A',
            _ => *x,
        }
    });
}
