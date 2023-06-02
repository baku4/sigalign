use crate::{Result, error_msg};
use std::fs::File;
use std::path::PathBuf;
use std::time::Instant;
use sigalign::{
    reference::{
        Reference,
        pattern_index::{
            PatternLocation,
            PatternIndex,
            ConcatenatedSequenceWithBoundaries,
            PatternIndexBuildError,
            lfi::{Lfi32B2V64, Lfi32B3V64, LfiOption},
        },
        sequence_storage::{
            SequenceStorage,
            in_memory::InMemoryStorage,
        },
        extensions::Serialize,
    },
    utils::FastaReader,
};

mod lfi_wrapper;

pub struct SigReferenceWrapper {
    pub inner: Reference<LfiWrapper, InMemoryStorage>,
}
impl AsRef<InnerReference> for SigReferenceWrapper {
    fn as_ref(&self) -> &InnerReference {
        &self.inner
    }
}
pub type InnerReference = Reference<LfiWrapper, InMemoryStorage>;
pub enum LfiWrapper {
    B2(Lfi32B2V64),
    B3(Lfi32B3V64),
}
impl PatternIndex for LfiWrapper {
    type Option = LfiOption;

    fn new(
        alignable_sequence: &[u8],
        concatenated_sequence_with_boundaries: ConcatenatedSequenceWithBoundaries,
        option: Self::Option,
    ) -> Result<Self, PatternIndexBuildError> {
        let chr_count = alignable_sequence.len();
        if chr_count <= 4 {
            Ok(Self::B2(
                Lfi32B2V64::new(alignable_sequence, concatenated_sequence_with_boundaries, option)?
            ))
        } else if chr_count <= 8 {
            Ok(Self::B3(
                Lfi32B3V64::new(alignable_sequence, concatenated_sequence_with_boundaries, option)?
            ))
        } else {
            Err(
                PatternIndexBuildError::OverMaximumCharacters { max: 8, input: chr_count as u32 }
            )
        }
    }
    fn locate(&self, pattern: &[u8], search_range: &Vec<u32>) -> Vec<PatternLocation> {
        match self {
            Self::B2(v) => v.locate(pattern, search_range),
            Self::B3(v) => v.locate(pattern, search_range),
        }
    }
}
const LFI_WRAPPER_B2_MAGIC_NUMBER: u8 = 0b00010011;
const LFI_WRAPPER_B3_MAGIC_NUMBER: u8 = 0b00100011;
impl Serialize for LfiWrapper {
    fn save_to<W>(&self, mut writer: W) -> std::result::Result<(), std::io::Error> where
        W: std::io::Write
    {
        match self {
            Self::B2(v) => {
                writer.write(&[LFI_WRAPPER_B2_MAGIC_NUMBER])?;
                v.save_to(writer)?;
            },
            Self::B3(v) => {
                writer.write(&[LFI_WRAPPER_B3_MAGIC_NUMBER])?;
                v.save_to(writer)?;
            },
        }
        Ok(())
    }
    fn load_from<R>(mut reader: R) -> std::result::Result<Self, std::io::Error> where
        R: std::io::Read,
        Self: Sized,
    {
        let mut buf = [0];
        reader.read_exact(&mut buf)?;
        match buf[0] {
            LFI_WRAPPER_B2_MAGIC_NUMBER => {
                let inner = Lfi32B2V64::load_from(reader)?;
                Ok(Self::B2(inner))
            },
            LFI_WRAPPER_B3_MAGIC_NUMBER => {
                let inner = Lfi32B3V64::load_from(reader)?;
                Ok(Self::B3(inner))
            },
            _ => panic!("") // TODO: Write err msg
        }
    }
}

impl SigReferenceWrapper {
    pub fn get_divided_sequence_storages(
        input_file_pathbuf: &PathBuf,
        use_rc: bool,
        maximum_size: Option<usize>,
    ) -> Result<Vec<InMemoryStorage>> {
        let mut sss = Vec::new();
        match maximum_size {
            None => {
                let mut ss = InMemoryStorage::new();
                ss.add_fasta_file(input_file_pathbuf)?;
                if use_rc {
                    ss.append_reverse_complement();
                }
                sss.push(ss);
            },
            Some(mut maximum_size) => {
                if use_rc {
                    maximum_size = maximum_size / 2;
                }
                let fasta_reader = FastaReader::from_path(input_file_pathbuf)?;
                let mut ss = InMemoryStorage::new();
                let mut current_size = 0;
                for (label, seq) in fasta_reader {
                    if current_size + seq.len() <= maximum_size {
                        ss.add_target(&seq, &label);
                        current_size += seq.len();
                    } else {
                        let mut to_swap_ss = InMemoryStorage::new();
                        std::mem::swap(&mut ss, &mut to_swap_ss);
                        if use_rc {
                            to_swap_ss.append_reverse_complement();
                        }
                        sss.push(to_swap_ss);
                        ss.add_target(&seq, &label);
                        current_size = seq.len();
                    }
                }
                sss.push(ss);
            },
        }
        Ok(sss)
    }
    pub fn build_and_save(
        sequence_storage: InMemoryStorage,
        // Pattern finder config
        kmer: Option<usize>,
        sa_sampling_ratio: Option<u64>,
        // Output
        file_path: &PathBuf,
    ) -> Result<()> {
        let start = Instant::now();
        let pattern_index_option = {
            let ltks = kmer.unwrap_or(9);
            let sasr = sa_sampling_ratio.unwrap_or(1);
            LfiOption {
                suffix_array_sampling_ratio: sasr,
                lookup_table_kmer_size: ltks as u32,
            }
        };
        let reference = InnerReference::new(
            sequence_storage,
            pattern_index_option,
        )?;
        eprint!("built in {} s; ", start.elapsed().as_secs_f64());
        let file = File::create(file_path)?;
        reference.save_to(file)?;
        Ok(())
    }
    pub fn load_from_file(file_path: &PathBuf) -> Result<Self> {
        let file = File::open(file_path)?;
        let inner = InnerReference::load_from(file)?;
        Ok(Self { inner })
    }
}
