use pyo3::exceptions::{
    PyFileExistsError, PyFileNotFoundError, PyOSError, PyTypeError, PyValueError,
};
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyList, PySequence, PyString, PyTuple, PyType};

use sigalign::{Reference, ReferenceBuilder};

use std::fs::File;
use std::path::Path;

#[pyclass(name = "Reference")]
pub struct PyReference {
    pub inner: Reference,
}

impl AsRef<Reference> for PyReference {
    fn as_ref(&self) -> &Reference {
        &self.inner
    }
}

#[pymethods]
impl PyReference {
    #[classmethod]
    #[pyo3(signature = (targets, set_uppercase=false, bases_to_ignore=""))]
    fn from_iterable(
        _cls: &Bound<PyType>,
        targets: &Bound<PyAny>,
        set_uppercase: bool,
        bases_to_ignore: &str,
    ) -> PyResult<Self> {
        let mut reference_builder = Self::new_configured_builder(set_uppercase, bases_to_ignore);

        reference_builder = add_targets_from_iterable_to_builder(reference_builder, targets)?;

        Self::from_builder(reference_builder)
    }
    #[classmethod]
    #[pyo3(signature = (fasta, set_uppercase=false, bases_to_ignore=""))]
    fn from_fasta(
        _cls: &Bound<PyType>,
        fasta: &Bound<PyAny>,
        set_uppercase: bool,
        bases_to_ignore: &str,
    ) -> PyResult<Self> {
        let fasta_bytes = if fasta.is_instance_of::<PyString>() {
            fasta.downcast::<PyString>()?.to_str()?.as_bytes()
        } else if fasta.is_instance_of::<PyBytes>() {
            fasta.downcast::<PyBytes>()?.as_bytes()
        } else {
            return Err(PyValueError::new_err(
                "The input must be either a string or bytes.",
            ));
        };

        let mut reference_builder = Self::new_configured_builder(set_uppercase, bases_to_ignore);

        reference_builder = add_fasta_bytes_to_builder(reference_builder, fasta_bytes)?;

        Self::from_builder(reference_builder)
    }
    #[classmethod]
    #[pyo3(signature = (file_path, set_uppercase=false, bases_to_ignore=""))]
    fn from_fasta_file(
        _cls: &Bound<PyType>,
        file_path: &Bound<PyAny>,
        set_uppercase: bool,
        bases_to_ignore: &str,
    ) -> PyResult<Self> {
        let mut reference_builder = Self::new_configured_builder(set_uppercase, bases_to_ignore);

        let file_path = file_path.downcast::<PyString>()?.to_str()?;
        reference_builder = add_fasta_file_to_builder(reference_builder, file_path)?;

        Self::from_builder(reference_builder)
    }

    #[getter(num_targets)]
    fn get_num_targets(&self) -> PyResult<u32> {
        Ok(self.inner.get_num_targets())
    }
    #[getter(estimated_size)]
    fn get_estimated_size(&self) -> PyResult<usize> {
        Ok(self.inner.get_estimated_size_in_bytes())
    }
    #[getter(total_length)]
    fn get_total_length(&self) -> PyResult<u32> {
        Ok(self.inner.get_total_length())
    }

    fn get_sequence(&self, target_index: u32) -> PyResult<String> {
        match self.inner.get_sequence(target_index) {
            Some(v) => Ok(String::from_utf8(v)?),
            None => Err(PyValueError::new_err("Target index is out of bound.")),
        }
    }
    fn get_label(&self, target_index: u32) -> PyResult<String> {
        match self.inner.get_label(target_index) {
            Some(v) => Ok(v),
            None => Err(PyValueError::new_err("Target index is out of bound.")),
        }
    }

    #[pyo3(signature = (file_path, overwrite=false))]
    fn save_to_file(&self, file_path: &str, overwrite: bool) -> PyResult<()> {
        if !overwrite {
            let path = Path::new(file_path);
            if path.exists() {
                return Err(PyFileExistsError::new_err(format!(
                    "The file '{}' already exists. Set 'overwrite' to true to allow replacing it.",
                    file_path
                )));
            }
        }

        let file = File::create(file_path)?;
        match self.inner.save_to(file) {
            Ok(_) => Ok(()),
            Err(e) => Err(PyOSError::new_err(format!(
                "Failed to save the reference to file '{}'. Error: {}",
                file_path, e
            ))),
        }
    }
    #[classmethod]
    fn load_from_file(_cls: &Bound<PyType>, file_path: &Bound<PyAny>) -> PyResult<Self> {
        let file_path = file_path.downcast::<PyString>()?.to_str()?;
        let file = match File::open(file_path) {
            Ok(v) => v,
            Err(e) => {
                return Err(PyFileNotFoundError::new_err(format!(
                    "The file '{}' could not be found. Error: {}",
                    file_path, e
                )))
            }
        };

        let inner = match Reference::load_from(file) {
            Ok(v) => v,
            Err(e) => {
                return Err(PyTypeError::new_err(format!(
                    "Failed to load a valid reference from file '{}'. Error: {}",
                    file_path, e
                )))
            }
        };

        Ok(Self { inner })
    }
}

impl PyReference {
    fn new_configured_builder(set_uppercase: bool, bases_to_ignore: &str) -> ReferenceBuilder {
        let mut reference_builder = ReferenceBuilder::new();
        if !bases_to_ignore.is_empty() {
            reference_builder = reference_builder.ignore_bases(bases_to_ignore.as_bytes());
        }
        reference_builder = reference_builder.set_uppercase(set_uppercase);
        reference_builder
    }
    fn from_builder(reference_builder: ReferenceBuilder) -> PyResult<Self> {
        let reference = match reference_builder.build() {
            Ok(v) => Ok(v),
            Err(e) => {
                let msg = format!("{e}");
                Err(PyValueError::new_err(msg))
            }
        }?;
        Ok(Self { inner: reference })
    }
}

#[inline]
fn add_targets_from_iterable_to_builder(
    mut reference_builder: ReferenceBuilder,
    targets: &Bound<PyAny>,
) -> PyResult<ReferenceBuilder> {
    let py_iterator = targets.iter()?;
    for value in py_iterator {
        let target = value?;
        reference_builder = add_target_to_builder(reference_builder, target)?;
    }
    Ok(reference_builder)
}

#[inline]
fn add_target_to_builder(
    mut reference_builder: ReferenceBuilder,
    target: Bound<PyAny>,
) -> PyResult<ReferenceBuilder> {
    if target.is_instance_of::<PyString>() {
        let sequence = target.downcast::<PyString>()?.to_str()?.as_bytes();
        reference_builder = reference_builder.add_target("", sequence);
    } else if target.is_instance_of::<PyBytes>() {
        let sequence = target.downcast::<PyBytes>()?.as_bytes();
        reference_builder = reference_builder.add_target("", sequence);
    } else if target.is_instance_of::<PyTuple>() || target.is_instance_of::<PyList>() {
        let py_sequence = target.downcast_into::<PySequence>().unwrap();
        if py_sequence.len()? == 2 {
            let first = py_sequence.get_item(0)?;
            let label = first.downcast::<PyString>()?.to_str()?;
            let second = py_sequence.get_item(1)?;
            let sequence = if second.is_instance_of::<PyString>() {
                second.downcast::<PyString>()?.to_str()?.as_bytes()
            } else if second.is_instance_of::<PyBytes>() {
                second.downcast::<PyBytes>()?.as_bytes()
            } else {
                return Err(PyTypeError::new_err("Each target must either be a sequence string or a list/tuple with length of 2 (containing label and sequence)."));
            };
            reference_builder = reference_builder.add_target(label, sequence);
        } else {
            return Err(PyValueError::new_err("Each target must either be a sequence string or a list/tuple with length of 2 (containing label and sequence)."));
        }
    } else {
        return Err(PyTypeError::new_err(
            "Targets must be of type str, bytes, tuple, or list",
        ));
    }
    Ok(reference_builder)
}

#[inline]
fn add_fasta_bytes_to_builder(
    mut reference_builder: ReferenceBuilder,
    fasta_bytes: &[u8],
) -> PyResult<ReferenceBuilder> {
    reference_builder = reference_builder.add_fasta(fasta_bytes).map_err(|e| {
        let msg = format!("{e}");
        PyValueError::new_err(msg)
    })?;
    Ok(reference_builder)
}
#[inline]
fn add_fasta_file_to_builder(
    mut reference_builder: ReferenceBuilder,
    file_path: &str,
) -> PyResult<ReferenceBuilder> {
    let file = File::open(file_path).map_err(|e| {
        let msg = format!("{e}");
        PyFileNotFoundError::new_err(msg)
    })?;
    reference_builder = reference_builder
        .add_fasta(file)
        .map_err(|_| PyValueError::new_err("Invalid FASTA record."))?;
    Ok(reference_builder)
}
