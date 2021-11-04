use anyhow::{Result, bail as error_msg};
use serde::{Serialize, Deserialize};
use pyo3::prelude::*;

mod reference;
mod aligner;

use reference::Reference;
use aligner::Aligner;

#[pymodule]
fn sigalign(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Reference>()?;
    m.add_class::<Aligner>()?;
    Ok(())
}