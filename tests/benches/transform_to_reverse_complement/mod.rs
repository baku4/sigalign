use criterion::{
    black_box, Criterion, BenchmarkId,
    PlotConfiguration, AxisScale,
};
use sigalign_tests::test_data_path::get_qry_for_val_path;
use sigalign::utils::FastaReader;
fn get_dna_seq_list() -> Vec<Vec<u8>> {
    let qry_for_val_path = get_qry_for_val_path();
    let fa_reader = FastaReader::from_path(qry_for_val_path).unwrap();
    fa_reader.into_iter().map(|(_, qry)| { qry }).collect()
}

pub fn transform_to_reverse_complement(c: &mut Criterion) {
    let mut group = c.benchmark_group("transform_to_reverse_complement");
    let dna_seq_list  = get_dna_seq_list();

    group.bench_function(
        "use_rev_iter",
        |b| b.iter(|| {
            use_rev_iter(black_box(&dna_seq_list));
        }
    ));

    group.bench_function(
        "use_comp_map",
        |b| b.iter(|| {
            use_comp_map(black_box(&dna_seq_list));
        }
    ));

}


const A_ASCII: u8 = 65;
const C_ASCII: u8 = 67;
const G_ASCII: u8 = 71;
const T_ASCII: u8 = 84;
const U_ASCII: u8 = 85;

#[inline]
fn use_rev_iter(sequences: &Vec<Vec<u8>>) -> Vec<Vec<u8>> {
    sequences.iter().map(|x| {
        x.iter().rev().map(|&character| {
            match character{
                A_ASCII => T_ASCII,
                C_ASCII => G_ASCII,
                G_ASCII => C_ASCII,
                T_ASCII => A_ASCII,
                _ => character,
            }
        }).collect()
    }).collect()
}
const COMP_MAP: [u8; 128] = [
    0u8, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31,
    32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63,
    64, T_ASCII, 66, G_ASCII, 68, 69, 70, C_ASCII, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, A_ASCII, 85, 86, 87, 88, 89, 90, 91, 92, 93, 94, 95,
    96, 97, 98, 99, 100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115, 116, 117, 118, 119, 120, 121, 122, 123, 124, 125, 126, 127,
];
#[inline]
fn use_comp_map(sequences: &Vec<Vec<u8>>) -> Vec<Vec<u8>> {
    sequences.iter().map(|x| {
        x.iter().rev().map(|&character| COMP_MAP[character as usize]).collect()
    }).collect()
}
