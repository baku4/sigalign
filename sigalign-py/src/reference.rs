use crate::{Result, error_msg};
use crate::{Serialize, Deserialize};

use pyo3::prelude::*;
use pyo3::types::*;
use pyo3::exceptions::PyException;

use sigalign::Reference as SigReference;
use sigalign::basic_sequence_provider::*;
use sigalign::reference::LtFmIndexConfig;

#[pyclass]
#[derive(Debug, Serialize, Deserialize)]
pub struct Reference {
    sig_reference_holder: SigReferenceHolder,
}

#[pymethods]
impl Reference {
    #[new]
    #[args(
        in_memory = "true",
        bwt_128 = "false",
        sampling_ratio = "2",
        lt_size = "0",
    )]
    fn new(
        fasta: &str,
        in_memory: bool,
        bwt_128: bool,
        sampling_ratio: u64,
        lt_size: usize,
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

        let sig_reference_holder = SigReferenceHolder::new(in_memory, lt_fm_index_config, fasta);

        match sig_reference_holder {
            Ok(sig_reference_holder) => Ok(Self { sig_reference_holder }),
            Err(err) => Err(PyException::new_err(format!("{}", err))),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
enum SigReferenceHolder {
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
    fn semi_global_alignment() -> String {

    }
    fn semi_global_alignment_without_label() -> String {

    }
}
