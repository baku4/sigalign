use pyo3::prelude::*;

mod aligner;
mod reference;
mod results;

use aligner::PyAligner;
use reference::PyReference;
use results::register_results_module_as_submodule;

#[pymodule]
fn sigalign(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyReference>()?;
    m.add_class::<PyAligner>()?;
    register_results_module_as_submodule(m)?;
    Ok(())
}
