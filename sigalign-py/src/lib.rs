use pyo3::prelude::*;

mod reference;
mod results;
mod aligner;


use reference::PyReference;
use results::register_results_module_as_submodule;
use aligner::PyAligner;

#[pymodule]
fn sigalign(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyReference>()?;
    m.add_class::<PyAligner>()?;
    register_results_module_as_submodule(m)?;
    Ok(())
}
