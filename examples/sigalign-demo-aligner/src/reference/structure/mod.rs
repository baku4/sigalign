use super::{Result, error_msg};

use sigalign::{ReferenceBuilder, Reference};
use sigalign::sequence_storage::{
    Divisible, SizeAware,
};
#[cfg(not(feature = "idx_fa"))]
use sigalign::sequence_storage::{
    InMemoryStorage as SeqStorage,
    InMemoryRcStorage as SeqRcStorage,
};
#[cfg(feature = "idx_fa")]
use sigalign::sequence_storage::{
    IndexedFastaStorage as SeqStorage,
    IndexedFastaRcStorage as SeqRcStorage,
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
    SeqStorages(Vec<SeqStorage>),
    SeqRcStorages(Vec<SeqRcStorage>),
}

impl SelfDescSeqPrvs {
    #[cfg(not(feature = "idx_fa"))]
    pub fn new(
        use_rc: bool,
        fasta_file: &PathBuf,
        divide_size: &Option<usize>,
    ) -> Result<Self> {
        if use_rc {
            let mut in_mem_rc_p = SeqRcStorage::new();
            in_mem_rc_p.add_fasta_file(fasta_file)?;
            let sps = match divide_size {
                None => vec![in_mem_rc_p],
                Some(max_length) => in_mem_rc_p.split_by_max_length(*max_length)?,
            };
            
            Ok(Self::SeqRcStorages(sps))
        } else {
            let mut in_mem_p = SeqStorage::new();
            in_mem_p.add_fasta_file(fasta_file)?;
            let sps = match divide_size {
                None => vec![in_mem_p],
                Some(max_length) => in_mem_p.split_by_max_length(*max_length)?,
            };
            Ok(Self::SeqStorages(sps))
        }
    }
    #[cfg(feature = "idx_fa")]
    pub fn new(
        use_rc: bool,
        fasta_file: &PathBuf,
        divide_size: &Option<usize>,
    ) -> Result<Self> {
        if use_rc {
            let sp = SeqRcStorage::new(fasta_file)?;
            let sps = vec![sp];
            
            Ok(Self::SeqRcStorages(sps))
        } else {
            let sp = SeqStorage::new(fasta_file)?;
            let sps = vec![sp];
            
            Ok(Self::SeqStorages(sps))
        }
    }
    pub fn splitted_size(&self) -> usize {
        match self {
            Self::SeqStorages(v) => v.len(),
            Self::SeqRcStorages(v) => v.len(),
        }
    }
}

pub enum SelfDescReference {
    Ref(Reference<SeqStorage>),
    RecRc(Reference<SeqRcStorage>),
}

impl SelfDescReference {    
    const REF_MAGIC_NUM: u8 = 11;
    const REF_RC_MAGIC_NUM: u8 = 22;

    pub fn build_and_save_to_file(
        reference_builder: &ReferenceBuilder,
        reference_paths: ReferencePaths,
        self_desc_seq_prv: SelfDescSeqPrvs,
    ) -> Result<()> {
        use std::time::Instant;

        match self_desc_seq_prv {
            SelfDescSeqPrvs::SeqStorages(sps) => {
                for (ref_idx, sp) in sps.into_iter().enumerate() {
                    eprintln!(" - Saving reference {}", ref_idx);
                    eprintln!("    Size:");
                    eprintln!("      Sequence storage: {}", sp.size_of());
                    let start_time = Instant::now();
                    let reference = reference_builder.clone().build(sp)?;
                    eprintln!("      Pattern finder: {}", reference.pattern_finder.size_of());
                    eprintln!("      Target record index: {}", reference.target_record_index.len() * 4);
                    let self_desc_ref = SelfDescReference::Ref(reference);
                    eprintln!("    Time elapsed to generate: {} s", start_time.elapsed().as_secs_f64());
                    let start_time = Instant::now();
                    self_desc_ref.save_to_file(&reference_paths.0[ref_idx])?;
                    eprintln!("    Time elapsed to save: {} s", start_time.elapsed().as_secs_f64());
                }
            },
            SelfDescSeqPrvs::SeqRcStorages(sps) => {
                for (ref_idx, sp) in sps.into_iter().enumerate() {
                    eprintln!(" - Saving reference {}", ref_idx);
                    eprintln!("    Size:");
                    eprintln!("      Sequence storage: {}", sp.size_of());
                    let start_time = Instant::now();
                    let reference = reference_builder.clone().build(sp)?;
                    eprintln!("      Pattern finder: {}", reference.pattern_finder.size_of());
                    eprintln!("      Target record index: {}", reference.target_record_index.len() * 4);
                    let self_desc_ref = SelfDescReference::RecRc(reference);
                    eprintln!("    Time elapsed to generate: {} s", start_time.elapsed().as_secs_f64());
                    let start_time = Instant::now();
                    self_desc_ref.save_to_file(&reference_paths.0[ref_idx])?;
                    eprintln!("    Time elapsed to save: {} s", start_time.elapsed().as_secs_f64());
                }
            },
        }
        
        Ok(())
    }
    fn magic_number(&self) -> u8 {
        match self {
            Self::Ref(_) => Self::REF_MAGIC_NUM,
            Self::RecRc(_) => Self::REF_RC_MAGIC_NUM,
        }
    }
    fn size_of(&self) -> usize {
        match self {
            Self::Ref(v) => v.size_of() + 1,
            Self::RecRc(v) => v.size_of() + 1,
        }
    }
    fn save_to_file(&self, out_path: &PathBuf) -> Result<()> {
        let mut out_file = File::create(out_path)?;
        out_file.set_len(self.size_of() as u64)?;

        match self {
            Self::Ref(inner_ref) => {
                out_file.write(&[self.magic_number()])?;
                inner_ref.save_to(out_file)?;
            },
            Self::RecRc(inner_ref) => {
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
            Self::REF_MAGIC_NUM => {
                let inner_ref = Reference::load_from(in_file)?;
                Ok(Self::Ref(inner_ref))
            },
            Self::REF_RC_MAGIC_NUM => {
                let inner_ref = Reference::load_from(in_file)?;
                Ok(Self::RecRc(inner_ref))
            },
            _ => error_msg!("Invalid reference file")
        }
    }
}
