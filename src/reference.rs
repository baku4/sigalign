pub use lt_fm_index::Config as FmConfig;
pub use lt_fm_index::FmIndex;

use crate::io::reader::{FastaRecords, fasta_records};

pub struct Reference {
    sequence_records: Vec<SequenceRecord>,
    accumulated_length: Vec<u64>,
    have_one_record: bool,
    lt_fm_index: Option<FmIndex>,
    /*    Options     */
    reverse_complement: bool,
    // Lt-fm-index
    klt_kmer: usize,
    sa_sampling_ratio: u64,
}

impl Reference {
    pub fn with_string() {
        
    }
    pub fn with_fasta_file(file_path: &str) {
        let records: FastaRecords = fasta_records(file_path);
        
    }
    pub fn load_index() {

    }
    pub fn generate_index() {

    }
}
struct SequenceRecord {
    label: String,
    sequence: Vec<u8>,
}

impl SequenceRecord {
    fn new() {

    }
}

enum StringRange {
    OnlyNc,
    NcWithN,
}

#[cfg(test)]
mod tests {
    use crate::*;
    use super::*;

    #[test]
    fn reference_test() {
        let records: FastaRecords = fasta_records("./src/tests/fasta/ERR209055.fa");
    }
}

/*
impl Debug for Reference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {}, {})", self.a, self.b, self.c)
    }
}

pub struct ReferenceDep<T: AsRef<[u8]>>{
    sequence: T,
    index: FmIndex
}
impl<T: AsRef<[u8]>> ReferenceDep<T> {
    pub fn new(sequence: T) -> Self {
        let fm_index =  Self::fmindex(&sequence);
        Self {
            sequence: sequence,
            index: fm_index,
        }
    }
    fn fmindex(sequence: &T) -> FmIndex {
        let seq = sequence.as_ref().iter().cloned().collect();
        // TODO: Custom fmindex configuration
        let fm_config: FmConfig = FmConfig::new()
            .set_kmer_lookup_table(FM_KLT_KMER_SIZE)
            .set_suffix_array_sampling_ratio(FM_SA_SAMPLING_RATIO);
        FmIndex::new(&fm_config, seq)
    }
}
*/