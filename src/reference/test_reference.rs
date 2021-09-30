use crate::core::{Query, Reference, RecordLocation};

use lt_fm_index::{FmIndex, LtFmIndexConfig, LtFmIndexAll, IO};

pub struct TestReference {
    fm_index_of_records: Vec<LtFmIndexAll>,
    length_of_records: Vec<usize>,
}

impl TestReference {
    pub fn new() -> Self {
        let ref_texts = vec![
            b"TTGGTGCGAGCTTCTCTCTGTCCGCATAGTCTTCCATGGTAAATCCCATAGAAACCTCCGCTACGATGACATCCTTCTGATATCTCTGTGAGAGATCATTCATATTATTGGTCAGATCATCAATGGTTCCGTGCCAGAAGGGATAATAGGACAGTCCGATGACATCGAAGTCCTCTCCTCTCTGTACATAGGCATCAAACCATTCTCTGCAGCGGGCATTGT".to_vec(),
            b"TTGCCTAAGCCTACGGATGCTCCGGCAGCCGACAGAACGAAGCCGATCTTGCCGGAAAATGAACTTCTGTTATTATTTTTGTGATTCATAGTTTAAACTCTCCCATAAAAAATTGTCGTAAACAGAATAAGTGTAACATAGAAATAAGGAAAATACAGTACGAAATAGTCTGATTGAGAATTGAAAAATTCGATAATGCAAGAGTGAAAATTGAATGGATGCATTTTAATTTTAATATAAAACAATAATTCTGAGGAAAAATA".to_vec(),
            b"ATCGGTATCTACCACATAGCCGTGGTTCTGAGAGGAAATATATACTCTTCCGGTCATCAGATCCTTTACCGGATGGTTGCCTCCTCTGTGTCCGTATTTCATTTTATGGGTATCTGCGCCGGTAGCCAGAGCCATCAGCTGATGTCCCAGACAGATTGCGAAAATAGGGATCTCGGTATCGTATAACTTTTTGATCTCGGCGATCACACCGGTACACTCCTTGGGGTCTCCCGGTCCGTTACTTAACATGATACCGTCGGGATTATCATCAATGATCTCCTGTGCTGGAGTTCCTGCGGGGTATACGGTCACCTCACAGCCTCTCTCTGCCAGGGATCTGGCGATATTTCTCTTGGCTCCCA".to_vec()
        ];
        
        let length_of_records: Vec<usize> = ref_texts.iter().map(|x| {
            x.len()
        }).collect();
        let fm_index_of_records = ref_texts.into_iter().map(|text| {
            LtFmIndexConfig::for_nucleotide().generate(text).unwrap()
        }).collect();

        Self {
            fm_index_of_records,
            length_of_records,
        }
    }
}

impl Reference for TestReference {
    fn locate(&self, pattern: Query, kmer: usize) -> Vec<RecordLocation> {
        let mut res: Vec<RecordLocation> = Vec::new();
        for (index, fm_index) in self.fm_index_of_records.iter().enumerate() {
            let location = fm_index.locate(pattern);
            let mut positions: Vec<usize> = location.into_iter().map(|v| v as usize).collect();
            positions.sort();

            if positions.len() != 0 {
                res.push(
                    RecordLocation {
                        index,
                        positions,
                    }
                )
            }
        }
        res
    }
    fn length_of_record(&self, record_index: usize) -> usize {
        self.length_of_records[record_index]
    }
}