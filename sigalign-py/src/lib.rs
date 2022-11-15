use anyhow::{Result, bail as error_msg};
use serde::{Serialize, Deserialize};
use pyo3::prelude::*;

mod sequence_storage;
mod reference;
mod result;
mod aligner;

use sequence_storage::SequenceStorage;
use reference::Reference;
use result::{
    QueryResult,
    FastaResult,
    register_result_module,
};
use aligner::Aligner;

#[pymodule]
#[pyo3(name = "sigalign")]
fn sigalign(py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<SequenceStorage>()?;
    m.add_class::<Reference>()?;
    m.add_class::<Aligner>()?;
    register_result_module(py, m)?;
    Ok(())
}
