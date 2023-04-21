use std::collections::{HashMap, BTreeMap};
//use serde::{Deserialize, Serialize};
//use serde_json::{Result, json};
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

#[cfg(test)]
mod tests {
    use std::{collections::{HashMap, BTreeMap}, f64::consts::PI, vec, fmt, path::Path, fs};
    use crate::{dxfextract::PolyLine, dxfwrite::savedxf};

    #[test]
    fn test_savedxf(){
        //Create sample data
        //BTreeMap
        let mut test_layers: BTreeMap<String, Vec<PolyLine>> = BTreeMap::new();

        //Random line
        let x1_values = vec![1.0, 2.0, 2.0, 2.0, 1.0]; 
        let y1_values = vec![1.0, 1.0, 2.0, 3.0, 3.0];
        let polyline1 = PolyLine::new(false, x1_values, y1_values);

        //Random line 2
        let x2_values = vec![1.0, 2.0, 3.0]; 
        let y2_values = vec![1.0, 1.0, 1.0];
        let polyline2 = PolyLine::new(false, x2_values, y2_values);

        //Insert lines into map
        test_layers.insert("layer1".to_string(), vec![polyline1.clone()]);
        test_layers.insert("layer2".to_string(), vec![polyline1.clone(), polyline2.clone()]);

        //Temporary output path
        let output_path = "test_save.dxf".to_string();

        //Check if save function works and that the file was created
        assert!(savedxf(test_layers, &output_path).is_ok());
        assert!(Path::new(&output_path).exists());

        //remove the temp file
        fs::remove_file(&output_path).unwrap();



        






    }

}