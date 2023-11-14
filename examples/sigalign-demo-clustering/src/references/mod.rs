use std::{path::PathBuf, fs::File};

use thiserror::Error;

use crate::core::Sequence;
use sigalign::reference::{
    Reference, ReferenceBuildError,
    pattern_index::lfi::{DynamicLfi, DynamicLfiOption},
    sequence_storage::in_memory::InMemoryStorage,
    extensions::Serialize,
};

type SigAlignReference = Reference<DynamicLfi, InMemoryStorage>;

pub struct SavedReferences {
    max_seq_len: u32,
    reference_paths: Vec<PathBuf>,
}

#[derive(Error, Debug)]
pub enum ReferenceError {
    #[error(transparent)]
    Saving(#[from] std::io::Error),
    #[error(transparent)]
    Building(#[from] ReferenceBuildError),
}

impl SavedReferences {
    pub fn build_and_save<I> (
        max_seq_len: u32,
        sorted_sequence_vector: Vec<Sequence>,
        uuid: &str,
        reference_root_dir: PathBuf,
    ) -> Result<Self, ReferenceError> where I: Iterator<Item = Sequence> {
        let mut reference_paths = Vec::new();

        let mut next_reference_index = 0;
        let mut sequence_storage = InMemoryStorage::new();
        let mut current_sequence_storage_length = 0;

        for sequence in sorted_sequence_vector {
            let new_seq_len = sequence.inner.len();
            if new_seq_len + current_sequence_storage_length <= max_seq_len as usize{
                let label = sequence.id.unwrap_or_default();
                sequence_storage.add_target(&label, &sequence.inner);
                current_sequence_storage_length += new_seq_len;
            } else {
                // Get sequence storage to build
                let mut to_build_sequence_storage = InMemoryStorage::new();
                std::mem::swap(&mut sequence_storage, &mut to_build_sequence_storage);
                current_sequence_storage_length = new_seq_len;
                // Build and save new reference
                let reference_path = get_reference_path(uuid, &reference_root_dir, &next_reference_index);
                build_and_save_reference_to_path(to_build_sequence_storage, &reference_path)?;
                reference_paths.push(reference_path);
                next_reference_index += 1;
            }
        }
        // Save last reference
        if current_sequence_storage_length != 0 {
            let reference_path = get_reference_path(uuid, &reference_root_dir, &next_reference_index);
            build_and_save_reference_to_path(sequence_storage, &reference_path)?;
            reference_paths.push(reference_path);
        }

        Ok(Self {
            max_seq_len,
            reference_paths,
        })
    }
}

fn get_reference_path(
    uuid: &str,
    reference_root_dir: &PathBuf,
    reference_index: &u32,
) -> PathBuf {
    let reference_file_name = format!("{}-{}", uuid, reference_index);
    let reference_path = reference_root_dir.join(reference_file_name);
    reference_path
}

fn build_and_save_reference_to_path(
    sequence_storage: InMemoryStorage,
    reference_path: &PathBuf,
) -> Result<(), ReferenceError> {
    let lfi_option = get_default_pattern_index_option();
    let reference: SigAlignReference = Reference::new(sequence_storage, lfi_option)?;
    let file = File::open(reference_path)?;
    reference.save_to(file)?;
    Ok(())
}

fn get_default_pattern_index_option() -> DynamicLfiOption {
    DynamicLfiOption {
        suffix_array_sampling_ratio: 2,
        max_lookup_table_byte_size: 10_000,
    }
}
