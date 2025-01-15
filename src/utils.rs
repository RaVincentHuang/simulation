use pyo3::prelude::*;

pub fn register_child_module(py: Python, parent_module: &PyModule, child_module: fn(Python, &PyModule) -> PyResult<()>, child_module_name: &str) -> PyResult<()> {
    let child_mod = PyModule::new(py, child_module_name)?;
    child_module(py, &child_mod)?;
    parent_module.add_submodule(child_mod)?;
    Ok(())
}
