use pyo3::exceptions::PyFileNotFoundError;
use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;
use pyo3::exceptions::{
    PyAttributeError,
    PyValueError,
    PyOSError,
    PyFileExistsError,
    PyException,
};

use super::{
    SequenceStorage,
};

use sigalign::{
    ReferenceBuilder as SigReferenceBuilder,
    Reference as SigReference,
};
use sigalign::sequence_storage::{
    InMemoryStorage as SigInMemoryStorage,
};

use std::fs::File;
use std::path::Path;

#[pyclass]
pub struct Reference {
    pub inner: SigReference<SigInMemoryStorage>,
}

#[pymethods]
impl Reference {
    #[new]
    #[args(
        comp_block = "false",
        klt_size = "None",
        sas_ratio = "None",
    )]
    fn new(
        sequence_storage: &SequenceStorage,
        comp_block: bool,
        klt_size: Option<usize>,
        sas_ratio: Option<u64>,
    ) -> PyResult<Self> {
        let mut builder = SigReferenceBuilder::new();
        // Kmer lookup table size
        if let Some(v) = klt_size {
            match builder.change_count_array_kmer(v) {
                Ok(v) => {
                    builder = v;
                },
                Err(e) => {
                    return Err(PyAttributeError::new_err(format!("{}", e)));
                },
            }
        }
        // Suffix array sampling ratio
        if let Some(v) = sas_ratio {
            match builder.change_sampling_ratio(v) {
                Ok(v) => {
                    builder = v;
                },
                Err(e) => {
                    return Err(PyAttributeError::new_err(format!("{}", e)));
                },
            }
        }
        // Compressed bwt block
        builder = if comp_block {
            builder.change_bwt_block_size_to_128()
        } else {
            builder.change_bwt_block_size_to_64()
        };
        // Build reference
        let inner = builder.build(sequence_storage.inner.clone());
        match inner {
            Ok(v) => {
                Ok(Self {
                    inner: v,
                })
            },
            Err(e) => {
                Err(PyException::new_err(format!("{}", e)))
            },
        }
    }

    #[getter(record_count)]
    fn get_record_count(&self) -> PyResult<usize> {
        let v = self.inner.total_record_count();
        Ok(v)
    }
    #[getter(klt_size)]
    fn get_lookup_table_kmer_size(&self) -> PyResult<usize> {
        let v = self.inner.get_lookup_table_kmer_size();
        Ok(v)
    }
    #[getter(sas_ratio)]
    fn get_suffix_array_sampling_ratio(&self) -> PyResult<u64> {
        let v = self.inner.get_suffix_array_sampling_ratio();
        Ok(v)
    }
    #[getter(bwt_block_size)]
    fn get_size_of_bwt_block(&self) -> PyResult<usize> {
        let v = self.inner.get_size_of_bwt_block();
        Ok(v)
    }
    #[getter(estimated_total_bytes)]
    fn get_estimated_total_bytes(&self) -> PyResult<usize> {
        let v = self.inner.size_of();
        Ok(v)
    }
    #[getter(searchable_characters)]
    fn get_searchable_characters(&self) -> PyResult<Vec<String>> {
        let v = self.inner.get_allowed_character_list();
        let vec: Vec<String> = v.iter().map(|v| char::from(*v).to_string()).collect();
        Ok(vec)
    }
    #[getter(is_nucleotide)]
    fn get_text_is_nucleotide(&self) -> PyResult<bool> {
        let v = self.inner.get_whether_text_type_is_nucleotide();
        Ok(v)
    }
    
    fn get_sequence(&self, record_index: usize) -> PyResult<String> {
        let optional_seq = self.inner.sequence_storage.get_sequence_safely(record_index);
        match optional_seq {
            Some(v) => {
                Ok(String::from_utf8(v).unwrap())
            },
            None => {
                Err(PyValueError::new_err("Record index is out of bound."))
            },
        }
    }
    fn get_label(&self, record_index: usize) -> PyResult<String> {
        let optional_label = self.inner.sequence_storage.get_label_safely(record_index);
        match optional_label {
            Some(v) => {
                Ok(v)
            },
            None => {
                Err(PyValueError::new_err("Record index is out of bound."))
            }
        }
    }

    #[args(
        overwrite = "false"
    )]
    fn to_file(
        &self,
        file_path: &str,
        overwrite: bool,
    ) -> PyResult<()> {
        if !overwrite {
            let path = Path::new(file_path);
            if path.exists() {
                return Err(PyFileExistsError::new_err(
                    "File path already exists. Enable overwrite if intended."
                ));
            }
        }

        let save_result = self.inner.save_to_file(file_path);
        match save_result {
            Ok(_) => Ok(()),
            Err(e) => Err(PyOSError::new_err(
                format!("Saving reference to file failed. ({})", e)
            ))
        }
    }
    #[staticmethod]
    fn from_file(
        file_path: &str,
    ) -> PyResult<Self> {
        let file = match File::open(file_path) {
            Ok(v) => v,
            Err(e) => return Err(PyFileNotFoundError::new_err(
                format!("File path does not exist. ({})", e)
            )),
        };

        let inner = match SigReference::<SigInMemoryStorage>::load_from(file) {
            Ok(v) => v,
            Err(e) => return Err(PyTypeError::new_err(
                format!("Invalid reference file. ({})", e)
            )),
        };

        Ok(Self { inner })
    }
}
