use std::collections::{HashMap, BTreeMap};
use serde::{Deserialize, Serialize};
use serde_json::{Result, json};
use pyo3::{PyResult, types::{PyModule, IntoPyDict}, PyAny, Python, Py};

use crate::dxfextract::PolyLine;

pub fn savedxf(map: BTreeMap<String, Vec<PolyLine>>, path: &String) -> PyResult<()> {
    let serialized = serde_json::to_string(&map).unwrap();
    let out_path = path.clone()/* .replace('.', "_").replace(' ', "_") + ".dxf"*/;
    Python::with_gil(|py| {
        let fun: Py<PyAny> = PyModule::from_code(
            py,
            include_str!("savedxf.py"),
            "savedxf.py",
            "savedxf",
        )?
        .getattr("savedxf")?
        .into();
        
        let mut kwargs = HashMap::<&str, &str>::new();
        kwargs.insert("json", &serialized);
        kwargs.insert("path", &out_path);
        fun.call(py, (), Some(kwargs.into_py_dict(py)))?;

        Ok(())
    })
}