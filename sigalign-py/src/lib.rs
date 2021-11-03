mod reference;
mod aligner;

use pyo3::prelude::*;

#[pymodule]
fn sigalign(_py: Python, m: &PyModule) -> PyResult<()> {

    Ok(())
}