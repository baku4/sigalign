use super::{Result, error_msg};

use sigalign::{ReferenceBuilder, Reference};
use sigalign::sequence_provider::{
    Divisible,
    InMemoryProvider, InMemoryRcProvider,
};

use std::fs::File;
use std::io::{Write, Read};
use std::path::PathBuf;

pub struct ReferencePaths(
    pub Vec<PathBuf>
);

impl ReferencePaths {
    pub fn new_to_save(
        ref_file_path: &PathBuf,
        divide_size: usize,
    ) -> Self {
        let reference_paths = (0..divide_size).map(|ref_idx| {
            if ref_idx == 0 {
                ref_file_path.clone()
            } else {
                let new_path = ref_file_path.clone();
                let mut file_name = ref_file_path.file_name().unwrap().to_os_string();
                file_name.push(format!("{}", ref_idx));
                new_path.with_file_name(file_name)
            }
        }).collect();
        
        Self(reference_paths)
    }
    pub fn new_for_load(
        ref_file_path: &PathBuf,
    ) -> Self {
        let mut reference_paths = vec![ref_file_path.clone()];

        let mut suffix_number = 1;
        loop {
            let mut suffix_added_file_name = ref_file_path.file_name().unwrap().to_os_string();
            suffix_added_file_name.push(format!("{}", suffix_number));

            let additional_ref_path = ref_file_path.clone().with_file_name(suffix_added_file_name);
            if additional_ref_path.exists() {
                reference_paths.push(additional_ref_path);
                suffix_number += 1;
            } else {
                break;
            }
        }

        Self(reference_paths)
    }
}

pub enum SelfDescSeqPrvs {
    InMemory(Vec<InMemoryProvider>),
    InMemoryRc(Vec<InMemoryRcProvider>),
}

impl SelfDescSeqPrvs {
    pub fn new(
        use_rc: bool,
        fasta_file: &PathBuf,
        divide_size: &Option<usize>,
    ) -> Result<Self> {
        if use_rc {
            let mut in_mem_rc_p = InMemoryRcProvider::new();
            in_mem_rc_p.add_fasta_file(fasta_file)?;
            let sps = match divide_size {
                None => vec![in_mem_rc_p],
                Some(max_length) => in_mem_rc_p.split_by_max_length(*max_length)?,
            };
            
            Ok(Self::InMemoryRc(sps))
        } else {
            let mut in_mem_p = InMemoryProvider::new();
            in_mem_p.add_fasta_file(fasta_file)?;
            let sps = match divide_size {
                None => vec![in_mem_p],
                Some(max_length) => in_mem_p.split_by_max_length(*max_length)?,
            };
            Ok(Self::InMemory(sps))
        }
    }
    pub fn splitted_size(&self) -> usize {
        match self {
            Self::InMemory(v) => v.len(),
            Self::InMemoryRc(v) => v.len(),
        }
    }
}

pub enum SelfDescReference {
    InMemory(Reference<InMemoryProvider>),
    InMemoryRc(Reference<InMemoryRcProvider>),
}

impl SelfDescReference {    
    const IN_MEM_MAGIC_NUM: u8 = 11;
    const IN_MEM_RC_MAGIC_NUM: u8 = 22;

    pub fn build_and_save_to_file(
        reference_builder: &ReferenceBuilder,
        reference_paths: ReferencePaths,
        self_desc_seq_prv: SelfDescSeqPrvs,
    ) -> Result<()> {
        match self_desc_seq_prv {
            SelfDescSeqPrvs::InMemory(sps) => {
                for (ref_idx, sp) in sps.into_iter().enumerate() {
                    let reference = reference_builder.clone().build(sp)?;
                    let self_desc_ref = SelfDescReference::InMemory(reference);

                    self_desc_ref.save_to_file(&reference_paths.0[ref_idx])?;
                }
            },
            SelfDescSeqPrvs::InMemoryRc(sps) => {
                for (ref_idx, sp) in sps.into_iter().enumerate() {
                    let reference = reference_builder.clone().build(sp)?;
                    let self_desc_ref = SelfDescReference::InMemoryRc(reference);

                    self_desc_ref.save_to_file(&reference_paths.0[ref_idx])?;
                }
            },
        }
        
        
        Ok(())
    }
    fn magic_number(&self) -> u8 {
        match self {
            Self::InMemory(_) => Self::IN_MEM_MAGIC_NUM,
            Self::InMemoryRc(_) => Self::IN_MEM_RC_MAGIC_NUM,
        }
    }
    fn size_of(&self) -> usize {
        match self {
            Self::InMemory(v) => v.size_of() + 1,
            Self::InMemoryRc(v) => v.size_of() + 1,
        }
    }
    fn save_to_file(&self, out_path: &PathBuf) -> Result<()> {
        let mut out_file = File::create(out_path)?;
        out_file.set_len(self.size_of() as u64)?;

        match self {
            Self::InMemory(inner_ref) => {
                out_file.write(&[self.magic_number()])?;
                inner_ref.save_to(out_file)?;
            },
            Self::InMemoryRc(inner_ref) => {
                out_file.write(&[self.magic_number()])?;
                inner_ref.save_to(out_file)?;
            },
        }

        Ok(())
    }
    pub fn load_from_file(in_path: &PathBuf) -> Result<Self> {
        let mut in_file = File::open(in_path)?;

        let mut magic_number: [u8; 1] = [0; 1];

        in_file.read_exact(&mut magic_number)?;

        match magic_number[0] {
            Self::IN_MEM_MAGIC_NUM => {
                let inner_ref = Reference::load_from(in_file)?;
                Ok(Self::InMemory(inner_ref))
            },
            Self::IN_MEM_RC_MAGIC_NUM => {
                let inner_ref = Reference::load_from(in_file)?;
                Ok(Self::InMemoryRc(inner_ref))
            },
            _ => error_msg!("Invalid reference file")
        }
    }
}
