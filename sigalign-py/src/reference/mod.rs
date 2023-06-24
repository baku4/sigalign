use pyo3::prelude::*;
use pyo3::exceptions::{
    PyFileNotFoundError,
    PyTypeError,
    PyKeyError,
    PyValueError,
    PyOSError,
    PyFileExistsError,
};
use pyo3::types::{
    PySequence, PyList, PyDict, PyTuple, PyString,
};

use sigalign::reference::{
    Reference,
    sequence_storage::{
        in_memory::{
            InMemoryStorage,
        },
    },
    pattern_index::{
        lfi::{
            DynamicLfi,
            DynamicLfiOption,
        },
    },
    extensions::{
        EstimateSize,
        Serialize,
    },
};

use std::fs::File;
use std::path::Path;

#[pyclass(name = "Reference")]
#[repr(transparent)]
pub struct PyReference {
    pub reference: Reference<DynamicLfi, InMemoryStorage>,
}

#[pymethods]
impl PyReference {
    #[new]
    fn py_new(
        targets: &PyAny,
        indexing_option: Option<&PyDict>,
    ) -> PyResult<Self> {
        let sequence_storage = get_sequence_storage_from_input_obj(targets)?;
        let pattern_index_option = parse_indexing_option(indexing_option)?;
        Self::new(sequence_storage, pattern_index_option)
    }

    #[getter(num_targets)]
    fn get_num_targets(&self) -> PyResult<u32> {
        Ok(self.reference.num_targets())
    }
    #[getter(estimated_size)]
    fn get_estimated_total_bytes(&self) -> PyResult<usize> {
        Ok(self.reference.serialized_size())
    }

    fn get_sequence(&self, target_index: u32) -> PyResult<String> {
        match self.reference.get_sequence(target_index) {
            Some(v) => Ok(String::from_utf8(v).unwrap()),
            None => Err(PyValueError::new_err("Target index is out of bound."))
        }
    }
    fn get_label(&self, target_index: u32) -> PyResult<String> {
        match self.reference.get_label(target_index) {
            Some(v) => Ok(v),
            None => Err(PyValueError::new_err("Target index is out of bound."))
        }
    }

    #[staticmethod]
    pub fn from_fasta_str(
        fasta_str: &str,
        indexing_option: Option<&PyDict>,
    ) -> PyResult<Self> {
        let sequence_storage = get_sequence_storage_from_fasta_str(fasta_str)?;
        let pattern_index_option = parse_indexing_option(indexing_option)?;
        Self::new(sequence_storage, pattern_index_option)
    }
    #[staticmethod]
    pub fn from_fasta_file(
        fasta_file: &str,
        indexing_option: Option<&PyDict>,
    ) -> PyResult<Self> {
        let sequence_storage = get_sequence_storage_from_fasta_file(fasta_file)?;
        let pattern_index_option = parse_indexing_option(indexing_option)?;
        Self::new(sequence_storage, pattern_index_option)
    }

    #[pyo3(signature = (file_path, overwrite=false))]
    fn save_to_file(
        &self,
        file_path: &str,
        overwrite: bool,
    ) -> PyResult<()> {
        if !overwrite {
            let path = Path::new(file_path);
            if path.exists() {
                
                return Err(PyFileExistsError::new_err(
                    format!("The file '{}' already exists. Set 'overwrite' to true to allow replacing it.", file_path)
                ));
            }
        }

        let file = File::create(file_path)?;
        match self.reference.save_to(file) {
            Ok(_) => Ok(()),
            Err(e) => Err(PyOSError::new_err(
                format!("Failed to save the reference to file '{}'. Error: {}", file_path, e)
            ))
        }
    }
    #[staticmethod]
    fn load_from_file(
        file_path: &str,
    ) -> PyResult<Self> {
        let file = match File::open(file_path) {
            Ok(v) => v,
            Err(e) => return Err(PyFileNotFoundError::new_err(
                format!("The file '{}' could not be found. Error: {}", file_path, e)
            )),
        };

        let reference = match Reference::load_from(file) {
            Ok(v) => v,
            Err(e) => return Err(PyTypeError::new_err(
                format!("Failed to load a valid reference from file '{}'. Error: {}", file_path, e)
            )),
        };

        Ok(Self { reference })
    }
}

impl PyReference {
    fn new(
        sequence_storage: InMemoryStorage,
        pattern_index_option: DynamicLfiOption,
    ) -> PyResult<Self> {
        let reference = match Reference::new(sequence_storage, pattern_index_option) {
            Ok(v) => Ok(v),
            Err(e) => {
                let msg = format!("{e}");
                Err(PyValueError::new_err(msg))
            },
        }?;
        Ok(Self { reference })
    }
}

fn get_sequence_storage_from_input_obj(ob: &PyAny) -> PyResult<InMemoryStorage> {
    let sequence_storage = if ob.is_instance_of::<PyList>() || ob.is_instance_of::<PyTuple>() {
        store_list_in_memory(ob)
    } else if ob.is_instance_of::<PyDict>() {
        store_dict_in_memory(ob)
    } else {
        let py_type = ob.get_type().name()?;
        match py_type {
            "range" => {
                store_list_in_memory(ob)
            },
            _ => {
                Err(PyTypeError::new_err("Targets must be of type dict, tuple, or list"))
            }
        }
    }?;
    Ok(sequence_storage)
}
fn get_sequence_storage_from_fasta_str(fasta_str: &str) -> PyResult<InMemoryStorage> {
    let mut sequence_storage = InMemoryStorage::new();
    sequence_storage.add_fasta_bytes(fasta_str.as_bytes());
    Ok(sequence_storage)
}
fn get_sequence_storage_from_fasta_file(fasta_file: &str) -> PyResult<InMemoryStorage> {
    let mut sequence_storage = InMemoryStorage::new();
    sequence_storage.add_fasta_file(fasta_file)?;
    Ok(sequence_storage)
}

fn store_list_in_memory(ob: &PyAny) -> PyResult<InMemoryStorage> {
    let mut in_memory_storage = InMemoryStorage::new();

    if ob.is_empty()? {
        Ok(in_memory_storage)
    } else {
        for item in ob.iter()? {
            let target = item?;
            match target.get_type().name()? {
                "str" => {
                    let sequence = target.str()?.to_str()?.as_bytes();
                    in_memory_storage.add_target("", sequence);
                },
                _ => {
                    let py_sequence = target.extract::<&PySequence>()?;
                    if py_sequence.len()? == 2 {
                        let label = py_sequence.get_item(0)?.str()?.to_str()?;
                        let sequence = py_sequence.get_item(1)?.str()?.to_str()?.as_bytes();
                        in_memory_storage.add_target(label, sequence);
                    } else {
                        return Err(PyValueError::new_err("Each target must either be a sequence string or a list/tuple with length of 2 (containing label and sequence)."))
                    }
                },
            }
        }
        Ok(in_memory_storage)
    }
}
fn store_dict_in_memory(ob: &PyAny) -> PyResult<InMemoryStorage> {
    let mut in_memory_storage = InMemoryStorage::new();

    if ob.is_empty()? {
        Ok(in_memory_storage)
    } else {
        let dict = ob.extract::<&PyDict>()?;
        
        for (k, v) in dict.iter() {
            let label = k.str()?.to_str()?;
            let sequence = v.str()?.to_str()?.as_bytes();
            in_memory_storage.add_target(label, sequence);
        }
        Ok(in_memory_storage)
    }
}
fn parse_indexing_option(ob: Option<&PyDict>) -> PyResult<DynamicLfiOption> {
    let mut option = DynamicLfiOption {
        suffix_array_sampling_ratio: 1,
        max_lookup_table_byte_size: 3_000_000,
    };
    if let Some(dict) = ob {
        for (k, v) in dict.iter() {
            let key_in_str = k.extract::<&PyString>()?.to_str()?;
            match key_in_str {
                "sasr" => {
                    option.suffix_array_sampling_ratio = v.extract::<u64>()?;
                },
                "lts" => {
                    option.max_lookup_table_byte_size = v.extract::<usize>()?;
                },
                _ => {
                    return Err(PyKeyError::new_err(
                        format!("Invalid key {} found.", key_in_str)
                    ))
                },
            };
        }
    }
    Ok(option)
}
