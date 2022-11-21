use crate::{Result, error_msg};
use std::fs::File;
use std::path::PathBuf;
use std::time::Instant;
use sigalign::{
    ReferenceBuilder as SigReferenceBuilder,
    Reference as SigReference,
};
use sigalign::sequence_storage::{
    InMemoryStorage as SigInMemoryStorage,
    Divisible,
};

pub struct Reference {
    pub inner: SigReference<SigInMemoryStorage>,
}

impl AsRef<InnerReference> for Reference {
    fn as_ref(&self) -> &InnerReference {
        &self.inner
    }
}

pub type InnerReference = SigReference<SigInMemoryStorage>;

impl Reference {
    pub fn get_divided_sequence_storages(
        input_file_pathbuf: &PathBuf,
        // Sequence storage type
        divide_size: Option<usize>,
        use_rc: bool,
    ) -> Result<Vec<SigInMemoryStorage>> {
        let mut ss = SigInMemoryStorage::new();
        ss.add_fasta_file(input_file_pathbuf)?;

        if use_rc {
            ss.append_reverse_complement();
        }

        let divided_ss = match divide_size {
            Some(v) => {
                ss.split_by_max_length(v)?
            },
            None => {
                vec![ss]
            },
        };

        Ok(divided_ss)
    }
    pub fn build_and_save(
        ss: SigInMemoryStorage,
        // Pattern finder config
        use_128_bwt: bool,
        kmer: Option<usize>,
        sa_sampling_ratio: Option<u64>,
        // Output
        file_path: &PathBuf,
    ) -> Result<()> {
        let start = Instant::now();
        let mut reference_builder = SigReferenceBuilder::new();
        if use_128_bwt {
            reference_builder = reference_builder.change_bwt_block_size_to_128();
        } else {
            reference_builder = reference_builder.change_bwt_block_size_to_64();
        }
        if let Some(v) = kmer {
            reference_builder = reference_builder.change_count_array_kmer(v)?;
        }
        if let Some(v) = sa_sampling_ratio {
            reference_builder = reference_builder.change_sampling_ratio(v)?;
        }

        let self_ = reference_builder.clone().build(ss).unwrap();
        eprint!("built in {} s; ", start.elapsed().as_secs_f64());
        let estimated_size = self_.size_of();
        eprintln!("about {} bytes", estimated_size);
        self_.save_to_file(file_path)
    }
    pub fn save_to_file(&self, file_path: &PathBuf) -> Result<()> {
        self.inner.save_to_file(file_path)
    }
    pub fn load_from_file(file_path: &PathBuf) -> Result<Self> {
        let file = File::open(file_path)?;
        let inner = SigReference::<SigInMemoryStorage>::load_from(file)?;
        Ok(Self { inner })
    }
    pub fn size_of(&self) -> usize {
        self.inner.size_of()
    }
}