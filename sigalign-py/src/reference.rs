use crate::{Result, error_msg};
use crate::{Serialize, Deserialize};

use pyo3::prelude::*;
use pyo3::types::*;
use pyo3::exceptions::PyException;
use sigalign::Reference as SigReference;
use sigalign::basic_sequence_provider::*;
use sigalign::reference::{LtFmIndexConfig, SequenceProvider, Writable};

use std::io::{Write, Read};
use std::fs::File;
use std::path::Path;
use std::ffi::OsString;
use std::convert::AsMut;

#[pyclass]
#[derive(Debug)]
pub struct Reference {
    pub sig_reference_holder: SigReferenceHolder,
}

#[pymethods]
impl Reference { 
    #[new]
    #[args(
        in_memory = "true",
        bwt_128 = "false",
        sampling_ratio = "2",
        lt_size = "0",
        reversed_nc = "false",
    )]
    fn new(
        fasta: &str,
        in_memory: bool,
        bwt_128: bool,
        sampling_ratio: u64,
        lt_size: usize,
        reversed_nc: bool,
    ) -> PyResult<Self> {
        let lt_fm_index_config = LtFmIndexConfig::new().change_sampling_ratio(sampling_ratio);
        let lt_fm_index_config = if bwt_128 {
            lt_fm_index_config.use_bwt_size_of_128()
        } else {
            lt_fm_index_config
        };
        let lt_fm_index_config = if lt_size > 1 {
            lt_fm_index_config.change_kmer_size_for_lookup_table(lt_size)
        } else {
            lt_fm_index_config
        };

        let sig_reference_holder = if reversed_nc {
            SigReferenceHolder::new_with_reversed(in_memory, lt_fm_index_config, fasta)
        } else {
            SigReferenceHolder::new(in_memory, lt_fm_index_config, fasta)
        };

        match sig_reference_holder {
            Ok(sig_reference_holder) => Ok(Self { sig_reference_holder }),
            Err(err) => Err(PyException::new_err(format!("{}", err))),
        }
    }
    
    #[args(
        overwrite = "false"
    )]
    fn save_to_file(
        &self,
        file: &str,
        overwrite: bool,
    ) -> PyResult<u64> {
        if !overwrite {
            let path = Path::new(file);
            if path.exists() {
                return Err(PyException::new_err(
                    "Path to save reference already exists."
                ));
            }
        }

        let mut file = File::create(file)?;
        
        // First 8 bytes is Tag
        let tag = self.sig_reference_holder.tag();
        file.write(&tag)?;

        match self.sig_reference_holder.save_to(&mut file) {
            Ok(_) => (),
            Err(err) => return Err(PyException::new_err(format!("{}", err))),
        }

        let size_of_file = file.metadata()?.len();

        Ok(size_of_file)
    }

    #[staticmethod]
    #[args(
        specified_fasta_path = "\"\""
    )]
    fn load_from_file(
        file: &str,
        specified_fasta_path: &str,
    ) -> PyResult<Self> {
        let mut opened_file = File::open(file)?;

        let mut tag: [u8; 8] = [0; 8];
        let tag_bytes = opened_file.read(&mut tag)?;

        if tag_bytes != 8 {
            return Err(PyException::new_err("Not a valid reference file"))
        }

        let sig_reference_holder = if tag == TAG_FOR_IN_MEMORY_REFERENCE {
            SigReferenceHolder::load_to_in_memory(opened_file)
        } else if tag == TAG_FOR_INDEXED_FASTA_REFERENCE {
            let fasta_path = match SigReferenceHolder::get_fasta_path(file, specified_fasta_path) {
                Ok(string) => string,
                Err(err) => return Err(PyException::new_err(format!("{}", err))),
            };

            let fasta_path_str = fasta_path.as_os_str().to_str().unwrap_or("");

            SigReferenceHolder::load_to_indexed_fasta(opened_file, fasta_path_str)
        } else {
            return Err(PyException::new_err("Not a valid reference file"))
        };

        match sig_reference_holder {
            Ok(sig_reference_holder) => {
                Ok(
                    Self {
                        sig_reference_holder
                    }
                )
            },
            Err(err) => {
                Err(PyException::new_err(format!("{}", err)))
            },
        }
    }
}

impl AsMut<SigReferenceHolder> for Reference {
    fn as_mut(&mut self) -> &mut SigReferenceHolder {
        &mut self.sig_reference_holder
    }
}

// Magic number for file format recognition
const TAG_FOR_IN_MEMORY_REFERENCE: [u8; 8] = [115, 97, 45, 105, 110, 109, 101, 109]; // sa-inmem
const TAG_FOR_INDEXED_FASTA_REFERENCE: [u8; 8] = [115, 97, 45, 105, 100, 120, 102, 97]; // sa-idxfa

#[derive(Debug)]
pub enum SigReferenceHolder {
    InMemory(SigReference<InMemoryProvider>),
    IndexedFasta(SigReference<IndexedFastaProvider>),
}
impl SigReferenceHolder {
    fn new(
        in_memory: bool,
        lt_fm_index_config: LtFmIndexConfig,
        fasta_file_path: &str,
    ) -> Result<Self> {
        Ok(
            if in_memory {
                let sequence_provider = InMemoryProvider::from_fasta_file(fasta_file_path)?;
                let sig_reference = SigReference::new_with_lt_fm_index_config(lt_fm_index_config, sequence_provider)?;
    
                Self::InMemory(sig_reference)
            } else {
                let sequence_provider = IndexedFastaProvider::new(fasta_file_path)?;
                let sig_reference = SigReference::new_with_lt_fm_index_config(lt_fm_index_config, sequence_provider)?;
    
                Self::IndexedFasta(sig_reference)
            }
        )
    }
    fn new_with_reversed(
        in_memory: bool,
        lt_fm_index_config: LtFmIndexConfig,
        fasta_file_path: &str,
    ) -> Result<Self> {
        if in_memory {
            let sequence_provider = InMemoryProvider::from_fasta_file_of_nucleotide_with_reverse_complement(fasta_file_path)?;
            let sig_reference = SigReference::new_with_lt_fm_index_config(lt_fm_index_config, sequence_provider)?;

            Ok(Self::InMemory(sig_reference))
        } else {
            error_msg!("Reverse complementary sequence can be used with only in-memory provider")
        }
    }
    // One byte tag to save the type information of sig-reference.
    fn tag(&self) -> [u8; 8] {
        match self {
            Self::InMemory(_) => TAG_FOR_IN_MEMORY_REFERENCE,
            Self::IndexedFasta(_) => TAG_FOR_INDEXED_FASTA_REFERENCE,
        }
    }
    fn save_to<W: Write>(&self, writer: W) -> Result<()> {
        match self {
            Self::InMemory(reference) => {
                reference.write_to(writer)
            },
            Self::IndexedFasta(reference) => {
                reference.write_to(writer)
            },
        }
    }
    fn load_to_in_memory<R: Read>(reader: R) -> Result<Self> {
        let sig_reference_in_memory = SigReference::<InMemoryProvider>::read_from(reader)?;

        Ok(Self::InMemory(sig_reference_in_memory))
    }
    fn load_to_indexed_fasta<R: Read>(reader: R, fasta_file_path: &str) -> Result<Self> {
        let sig_reference_indexed_fasta = SigReference::<IndexedFastaProvider>::read_from(reader, fasta_file_path)?;

        Ok(Self::IndexedFasta(sig_reference_indexed_fasta))
    }
    fn get_fasta_path(
        file: &str,
        specified_fasta_path: &str,
    ) -> Result<OsString> {
        if specified_fasta_path == "" {
            // Try to infer
            // with extension '.fa', '.fna', or '.fasta'
            // in same directory with reference
            let path = Path::new(file);

            for extension in ["fa", "fna", "fasta"].iter() {
                let inferred_path = path.with_extension(extension);
                if inferred_path.exists() {
                    return Ok(inferred_path.into_os_string())
                }
            }

            error_msg!("Cannot find fasta file.")
        } else {
            let path = Path::new(specified_fasta_path);

            if path.exists() {
                Ok(path.as_os_str().to_os_string())
            } else {
                error_msg!("Input fasta file is not exist.")
            }
        }
    }
}
