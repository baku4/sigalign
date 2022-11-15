use pyo3::prelude::*;
use pyo3::exceptions::{
    PyFileNotFoundError,
    PyValueError,
};

use sigalign::sequence_storage::{
    SequenceStorage as SigSequenceStorage,
    InMemoryStorage as SigInMemoryStorage,
};

#[pyclass]
#[derive(Debug, Clone)]
pub struct SequenceStorage {
    pub inner: SigInMemoryStorage,
}

#[pymethods]
impl SequenceStorage {
    #[new]
    fn py_new() -> Self {
        let inner = SigInMemoryStorage::new();
        Self { inner }
    }
    fn __repr__(&self) -> String {
        format!("InMemorySequenceStorage(num records: {})", self.inner.total_record_count())
    }
    #[getter(record_count)]
    fn record_count(&self) -> PyResult<usize> {
        Ok(self.inner.total_record_count())
    }

    fn add_record(
        &mut self,
        label: &str,
        sequence: &str,
    ) -> PyResult<()> {
        self.inner.add_record(
            sequence.as_bytes(),
            label,
        );
        Ok(())
    }
    fn add_fasta_str(
        &mut self,
        fasta_str: &str,
    ) -> PyResult<()> {
        self.inner.add_fasta_bytes(fasta_str.as_bytes());
        Ok(())
    }
    fn add_fasta_file(
        &mut self,
        fasta_file: &str,
    ) -> PyResult<()> {
        match self.inner.add_fasta_file(fasta_file) {
            Ok(_) => Ok(()),
            Err(e) => Err(PyFileNotFoundError::new_err(format!("{}", e))),
        }
    }

    fn get_sequence(&self, record_index: usize) -> PyResult<String> {
        let optional_seq = self.inner.get_sequence_safely(record_index);
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
        let optional_label = self.inner.get_label_safely(record_index);
        match optional_label {
            Some(v) => {
                Ok(v)
            },
            None => {
                Err(PyValueError::new_err("Record index is out of bound."))
            }
        }
    }
}
