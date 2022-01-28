use super::{Result, error_msg};

use sigalign::Reference;
use sigalign::sequence_provider::{
    InMemoryProvider, InMemoryRcProvider, IndexedFastaProvider, IndexedFastaRcProvider,
};

use std::fs::File;
use std::io::{Write, Read};
use std::path::PathBuf;

const IN_MEM_MAGIC_NUM: u8 = 11;
const IN_MEM_RC_MAGIC_NUM: u8 = 22;
const IDX_FA_MAGIC_NUM: u8 = 33;
const IDX_FA_RC_MAGIC_NUM: u8 = 44;

pub enum SelfDescReference {
    InMemory(Reference<InMemoryProvider>),
    InMemoryRc(Reference<InMemoryRcProvider>),
    IndexedFasta(Reference<IndexedFastaProvider>),
    IndexedFastaRc(Reference<IndexedFastaRcProvider>),
}

impl SelfDescReference {
    fn magic_number(&self) -> u8 {
        match self {
            Self::InMemory(_) => IN_MEM_MAGIC_NUM,
            Self::InMemoryRc(_) => IN_MEM_RC_MAGIC_NUM,
            Self::IndexedFasta(_) => IDX_FA_MAGIC_NUM,
            Self::IndexedFastaRc(_) => IDX_FA_RC_MAGIC_NUM,
        }
    }
    pub fn save_to_file(&self, out_path: &PathBuf) -> Result<()> {
        let mut out_file = File::create(out_path)?;

        match self {
            Self::InMemory(inner_ref) => {
                out_file.write(&[self.magic_number()])?;
                inner_ref.save_to(out_file)?;
            },
            Self::InMemoryRc(inner_ref) => {
                out_file.write(&[self.magic_number()])?;
                inner_ref.save_to(out_file)?;
            },
            Self::IndexedFasta(inner_ref) => {
                out_file.write(&[self.magic_number()])?;
                inner_ref.save_to(out_file)?;
            },
            Self::IndexedFastaRc(inner_ref) => {
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
            IN_MEM_MAGIC_NUM => {
                let inner_ref = Reference::load_from(in_file)?;
                Ok(Self::InMemory(inner_ref))
            },
            IN_MEM_RC_MAGIC_NUM => {
                let inner_ref = Reference::load_from(in_file)?;
                Ok(Self::InMemoryRc(inner_ref))
            },
            IDX_FA_MAGIC_NUM => {
                let inner_ref = Reference::load_from(in_file)?;
                Ok(Self::IndexedFasta(inner_ref))
            },
            IDX_FA_RC_MAGIC_NUM => {
                let inner_ref = Reference::load_from(in_file)?;
                Ok(Self::IndexedFastaRc(inner_ref))
            },
            _ => error_msg!("Invalid reference file")
        }
    }
}