const A_UTF8: u8 = 65;
const C_UTF8: u8 = 67;
const G_UTF8: u8 = 71;
const T_UTF8: u8 = 84;

pub fn get_reverse_complement(sequence: &[u8]) -> Vec<u8> {
    let mut rc_seq: Vec<u8> = Vec::with_capacity(sequence.len());
    sequence.iter().for_each(|chr| {
        match *chr {
            A_UTF8 => rc_seq.push(T_UTF8),
            C_UTF8 => rc_seq.push(G_UTF8),
            G_UTF8 => rc_seq.push(C_UTF8),
            T_UTF8 => rc_seq.push(A_UTF8),
            _ => rc_seq.push(*chr),
        }
    });
    rc_seq
}