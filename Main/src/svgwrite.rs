
use dxfextract::PolyLine;
use svg::node::element as svg_element;
use svg::Document;
use std::{collections::{HashMap, BTreeMap}};
use log::{info};
use crate::dxfextract;
use pyo3::{PyResult, types::{PyModule, IntoPyDict}, PyAny, Python, Py};
pub fn create_svg(layer_polylines: &BTreeMap<String, Vec<PolyLine>>, min_x: &f64, max_y: &f64, width: &f64, height: &f64, colors: Vec<[f32; 3]>) -> Document{
    let mut document = Document::new()
    // .set::<_, (f64, f64, f64, f64)>("viewBox", (22000.0, 90000.0, 2800.0, 4000.0))
        .set::<_, (f64, f64, f64, f64)>("viewBox", (0.0, 0.0, width.clone(), height.clone()))
        .set(
        "xmlns:inkscape",
        "http://www.inkscape.org/namespaces/inkscape",
    )
        .set("inkscape:version", "1.1.1 (3bf5ae0d25, 2021-09-20)");
    let line_width: f64 = height / 1000.0;
    let mut line_width_string = line_width.to_string();
    line_width_string.push_str("px");
    


    // insert polylines into svg paths

    //Colors to use when creating layers. Priority from right to left
    //let mut colors = vec!["purple(16)","navy(16)","seagreen","darkslategrey","black","darkorchid","indianred","darkolivegreen","forestgreen", "indigo", "pink", "olive", "lightsalmon", "cornflowerblue", "deepskyblue", "brown", "darkred", "chocolate", "blueviolet", "purple", "orange", "green", "blue", "red"];
    for ((name, polylines), _color) in layer_polylines.iter().zip(colors.iter()) {
        //Uses the next color for this layer. If none are left use black
        /*let color = 
        match colors.pop(){
            None => "black",
            Some(c) => c,
        };*/
        //let _color = colors.pop().unwrap();
        let color = format!("rgba({},{},{},1.0)", (_color[0] * 255.) as i32, (_color[1] * 255.) as i32, (_color[2]  * 255.) as i32);
        //println!("{}", &color);
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
                .set("stroke", color.as_str())
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



#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    //NEED TO CHECK IF FORMATTING OF SVG DOCUMENT IS CORRECT
    fn test_create_svg(){
        let mut layer_polylines: BTreeMap<String, Vec<PolyLine>> = BTreeMap::new();

        let x1_values = vec![0.0, 1.0, 2.0]; 
        let y1_values = vec![0.0, 1.0, 0.0];
        let polyline1 = PolyLine::new(true, x1_values, y1_values);

        layer_polylines.insert("layer1".to_string(), vec![polyline1]);
        let min_x = 0.0;
        let max_y = 2.0;
        let width = 100.0;
        let height = 100.0;

        let test_doc = create_svg(&layer_polylines, &min_x, &max_y, &width, &height, vec![[255., 0., 0.]]);

        assert_eq!(test_doc.to_string(), 
        "<svg inkscape:version=\"1.1.1 (3bf5ae0d25, 2021-09-20)\" viewBox=\"0 0 100 100\" xmlns=\"http://www.w3.org/2000/svg\" xmlns:inkscape=\"http://www.inkscape.org/namespaces/inkscape\">\n<g inkscape:groupmode=\"layer\" inkscape:label=\"layer1\" style=\"display:inline\">\n<path d=\"M0,2 L1,1 L2,2 z\" fill=\"none\" stroke=\"red\" stroke-width=\"0.1px\"/>\n</g>\n</svg>"
    );
    
        

        
    }

}