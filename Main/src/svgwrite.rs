
use dxfextract::PolyLine;
use svg::node::element as svg_element;
use svg::Document;
use std::{collections::{HashMap, BTreeMap}};
use log::{info};
use crate::dxfextract;
use pyo3::{PyResult, types::{PyModule, IntoPyDict}, PyAny, Python, Py};
pub fn create_svg(layer_polylines: &BTreeMap<String, Vec<PolyLine>>, min_x: &f64, max_y: &f64, width: &f64, height: &f64) -> Document{
    let mut document = Document::new()
    // .set::<_, (f64, f64, f64, f64)>("viewBox", (22000.0, 90000.0, 2800.0, 4000.0))
        .set::<_, (f64, f64, f64, f64)>("viewBox", (0.0, 0.0, width.clone(), height.clone()))
        .set(
        "xmlns:inkscape",
        "http://www.inkscape.org/namespaces/inkscape",
    )
        .set("inkscape:version", "1.1.1 (3bf5ae0d25, 2021-09-20)");
    let line_width: f64 = (height/1000.0);
    let mut line_width_string = line_width.to_string();
    line_width_string.push_str("px");
    


    // insert polylines into svg paths

    //Colors to use when creating layers. Priority from right to left
    let mut colors = vec!["purple(16)","navy(16)","seagreen","darkslategrey","black","darkorchid","indianred","darkolivegreen","forestgreen", "indigo", "pink", "olive", "lightsalmon", "cornflowerblue", "deepskyblue", "brown", "darkred", "chocolate", "blueviolet", "purple", "orange", "green", "blue", "red"];
    for (name, polylines) in layer_polylines.iter() {
        //Uses the next color for this layer. If none are left use black
        let color = 
        match colors.pop(){
            None => "black",
            Some(c) => c,
        };
        let mut group = svg_element::Group::new()
            .set("inkscape:label", name.as_str())
            .set("inkscape:groupmode", "layer")
            .set("style", "display:inline");
        for polyline in polylines.iter() {
            let mut path_data = svg_element::path::Data::new();
            let x_values = polyline.x_values.iter();
            let y_values = polyline.y_values.iter();
            let mut xy_values = x_values.zip(y_values).map(|(x, y)| (x - min_x, y - max_y));

            if let Some((x, y)) = xy_values.next() {
                path_data = path_data.move_to((x, -y));
            }

            for (x, y) in xy_values {
                path_data = path_data.line_to((x, -y));
            }

            if polyline.is_closed {
                path_data = path_data.close();
            }
            

            let path = svg_element::Path::new()
                .set("fill", "none")
                .set("stroke", color)
                .set("stroke-width", line_width_string.as_str())
                .set("d", path_data);

            group = group.add(path);
        }

        document = document.add(group);

        info!("created svg layer: {}", name);
    }
    document
}
pub fn save_svg(path: &String, file: &Document){
    match svg::save(path.clone() /* .replace('.', "_").replace(' ', "_") + ".svg"*/, file) {
        Ok(_) => info!("Saved SVG: {}", path),
        Err(err) => panic!("Error: {}", err),
    };
}
//alternative working way of saving as svg
pub fn _save_svg_ez(path: &String) -> PyResult<()>{
    let out_path = path.clone().replace('.', "_").replace(' ', "_") + ".svg";
    Python::with_gil(|py| {
        let fun: Py<PyAny> = PyModule::from_code(
            py,
            include_str!("savedxf.py"),
            "savedxf.py",
            "savedxf",
        )?
        .getattr("savesvg")?
        .into();
        
        let mut kwargs = HashMap::<&str, &str>::new();
        kwargs.insert("in_path", &path);
        kwargs.insert("out_path", &out_path);
        fun.call(py, (), Some(kwargs.into_py_dict(py)))?;

        Ok(())
    })
}
