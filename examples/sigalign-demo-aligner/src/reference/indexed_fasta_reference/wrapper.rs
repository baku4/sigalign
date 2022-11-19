use crate::{Result, error_msg};
use std::fs::File;
use std::path::PathBuf;

use sigalign::{
    ReferenceBuilder as SigReferenceBuilder,
    Reference as SigReference,
};
use sigalign::sequence_storage::{
    IndexedFastaStorage as SigIndexedFastaStorage,
    Divisible,
};

pub struct Reference {
    pub inner: SigReference<SigIndexedFastaStorage>,
}

impl AsRef<InnerReference> for Reference {
    fn as_ref(&self) -> &InnerReference {
        &self.inner
    }
}

pub type InnerReference = SigReference<SigIndexedFastaStorage>;

impl Reference {
    pub fn build(
        input_file_pathbuf: &PathBuf,
        // Pattern finder config
        use_128_bwt: bool,
        kmer: Option<usize>,
        sa_sampling_ratio: Option<u64>,
    ) -> Result<Vec<Self>> {
        eprintln!(" Indexed fasta storage is used.");
        eprintln!("  - Reverse complementary is not handled in reference");
        eprintln!("  - Reference can not divided into several chunks.");
        let ss = SigIndexedFastaStorage::new(input_file_pathbuf)?;

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

        Ok(
            vec![Self {
                inner: reference_builder.build(ss).unwrap()
            }]
        )
    }
    pub fn save_to_file(&self, file_path: &PathBuf) -> Result<()> {
        self.inner.save_to_file(file_path)
    }
    pub fn load_from_file(file_path: &PathBuf) -> Result<Self> {
        let file = File::open(file_path)?;
        let inner = SigReference::<SigIndexedFastaStorage>::load_from(file)?;
        Ok(Self { inner })
    }
    pub fn size_of(&self) -> usize {
        self.inner.size_of()
    }
}