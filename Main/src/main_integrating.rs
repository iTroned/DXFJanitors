/*
    A simple CLI that takes as input a dxf file path and produces an svg according to laiouts standards
    This CLI uses clap::Parser, so it is nicely self documented, try --help.
    Whenever possible, this program tries to produce closed paths, however that is not always possible.
*/

// TODO: do something better than hardcoding this.
const NUM_SEGMENTS: usize = 16;

use clap::Parser;

use dxf::entities as dxfe;
use std::{collections::HashMap, f64::consts::PI};
use svg::node::element as svg_element;

use log::{error, info, warn};
use uuid::Uuid;
use warp::{
    http::StatusCode,
    multipart::{FormData, Part},
    Filter, Rejection, Reply,
};

#[derive(Parser, Debug)]
struct Args {
    /// Path to input file is required
    #[clap(short = 'i', long = "input")]
    input: String,
    /// Path to output file, may be omitted.
    #[clap(short = 'o', long = "output")]
    output: Option<String>,
}

#[derive(Clone, Default)]
pub struct PolyLine {
    is_closed: bool,
    x_values: Vec<f64>,
    y_values: Vec<f64>,
    stroke: String,
}

impl From<dxfe::Line> for PolyLine {
    fn from(e: dxfe::Line) -> Self {
        Self {
            x_values: vec![e.p1.x, e.p2.x],
            y_values: vec![e.p1.y, e.p2.y],
            stroke: "black".into(),
            is_closed: false,
        }
    }
}

impl From<dxfe::LwPolyline> for PolyLine {
    fn from(e: dxfe::LwPolyline) -> Self {
        Self {
            x_values: e.vertices.iter().map(|v| v.x).collect(),
            y_values: e.vertices.iter().map(|v| v.y).collect(),
            stroke: "black".into(),
            is_closed: e.get_is_closed(),
        }
    }
}

impl From<dxfe::Circle> for PolyLine {
    fn from(e: dxfe::Circle) -> Self {
        make_polyline_circle(NUM_SEGMENTS, &e)
    }
}

impl From<dxfe::Arc> for PolyLine {
    fn from(e: dxfe::Arc) -> Self {
        make_polyline_arc(NUM_SEGMENTS, &e)
    }
}

impl From<dxfe::Ellipse> for PolyLine {
    fn from(e: dxfe::Ellipse) -> Self {
        make_polyline_ellipse(NUM_SEGMENTS, &e)
    }
}
#[tokio::main]
async fn main() {
    // load logger from environment
    env_logger::init_from_env(
        env_logger::Env::new()
            .filter("LOG")
            .write_style("LOG_STYLE"),
    );
    let upload_route = warp::path("upload")
        .and(warp::post())
        .and(warp::multipart::form().max_length(100_000_000))//Size of acceptable files, can be altered. In bytes
        .and_then(upload);
    let download_route = warp::path("files").and(warp::fs::dir("src/files/"));

    let router = upload_route.or(download_route).recover(handle_rejection);
    println!("Server started at localhost:8080");
    warp::serve(router).run(([0, 0, 0, 0], 8080)).await;
}

async fn upload(form: FormData) -> Result<impl Reply, Rejection> {
    let parts: Vec<Part> = form.try_collect().await.map_err(|e| {
        eprintln!("form error: {}", e);
        warp::reject::reject()
    })?;
    let tag = Uuid::new_v4().to_string();
    for p in parts {
        if p.content_type() == Some("application/octet-stream") {
            if p.name() == "file" {
                let value = p
                    .stream()
                    .try_fold(Vec::new(), |mut vec, data| {
                        vec.put(data);
                        async move { Ok(vec) }
                    })
                    .await
                    .map_err(|e| {
                        eprintln!("reading file error: {}", e);
                        warp::reject::reject()
                    })?;

                let file_name = format!("src/files/{}.dxf", tag);
                //let file_name = format!("files/test.dxf");
                tokio::fs::write(&file_name, value).await.map_err(|e| {
                    eprint!("error writing file: {}", e);
                    warp::reject::reject()
                })?;
                println!("created file: {}", file_name);
                //Starts a thread converting the file
                thread::spawn(|| {
                    start_conversion(file_name);
                });
            }
        }
        else {
            warp::reject::reject();
        }
    }
    
    Ok(tag)
}
fn start_conversion(path: String){
    let dxf_file = dxf::Drawing::load_file(path).expect("expexted valid input file");
    println!("amount of ucs: {}", dxf_file.ucss().count());
    let output_path = args
    .output
    .unwrap_or(path.clone().replace('.', "_").replace(' ', "_") + "_export.svg");


    //Creates and saves svg file
    let layers = extract_layers(&dxf_file);

    write_layers_to_svg(&layers, output_path);

    //Removes the DXF from server
    fs::remove_file(path).expect("File delete failed");
}
//dxf file read from the path provided as a parameter
fn read_dxf(_path: &String, _dxf_file: &mut Drawing) -> dxf::DxfResult<()> {
    *_dxf_file = Drawing::load_file(_path)?;
    Ok(())
}
// create this struct since unable to find Layer struct in dxf lib that also contains entities
#[derive(Clone, Debug, Default)]
struct Layer {
    name: String,
    data: LayerData,
}

#[derive(Clone, Debug, Default)]
struct LayerData {
    lines: Vec<dxfe::Line>,
    lw_polylines: Vec<dxfe::LwPolyline>,
    arcs: Vec<dxfe::Arc>,
    circles: Vec<dxfe::Circle>,
    ellipses: Vec<dxfe::Ellipse>,
}

impl Layer {
    fn new(name: String) -> Self {
        Self {
            name,
            ..Default::default()
        }
    }

    fn num_entities(&self) -> usize {
        let d = &self.data;
        d.lines.len() + d.lw_polylines.len() + d.arcs.len() + d.circles.len() + d.ellipses.len()
    }

    fn into_polylines(&self) -> Vec<PolyLine> {
        let mut polylines = Vec::<PolyLine>::default();
        let d = &self.data;
        polylines.append(&mut d.lines.iter().cloned().map(|e| e.into()).collect());
        polylines.append(&mut d.lw_polylines.iter().cloned().map(|e| e.into()).collect());
        polylines.append(&mut d.arcs.iter().cloned().map(|e| e.into()).collect());
        polylines.append(&mut d.circles.iter().cloned().map(|e| e.into()).collect());
        polylines.append(&mut d.ellipses.iter().cloned().map(|e| e.into()).collect());
        polylines
    }
}

fn extract_layers(dxf_file: &dxf::Drawing) -> HashMap<String, Layer> {
    let mut layers = HashMap::<String, Layer>::default();

    // initialize layers
    info!("initializing {} layers", dxf_file.layers().count());
    for dxf_layer in dxf_file.layers() {
        let name = dxf_layer.name.clone();
        layers.insert(name.clone(), Layer::new(name));
    }

    let mut unhandled = HashMap::<String, usize>::default();

    // loading layer data
    for (_, layer) in layers.iter_mut() {
        for entity in dxf_file.entities() {
            let layer_name = entity.common.layer.to_string();
            if layer_name != layer.name {
                continue;
            }
            let ld = &mut layer.data;

            let dxf_type = &entity.specific;

            use dxfe::EntityType as ET;

            match dxf_type {
                // handled entities
                ET::Line(e) => ld.lines.push(e.clone()),
                ET::LwPolyline(e) => ld.lw_polylines.push(e.clone()),
                ET::Arc(e) => ld.arcs.push(e.clone()),
                ET::Circle(e) => ld.circles.push(e.clone()),
                ET::Ellipse(e) => ld.ellipses.push(e.clone()),
                ET::Spline(e) => error!("unhandled spline {:?}", e),

                // unhandled entities --->
                e => {
                    let dxf_type_name = format!("{:?}", e).split('(').next().unwrap().to_owned();

                    if unhandled.contains_key(&dxf_type_name) == false {
                        unhandled.insert(dxf_type_name.clone(), 0);
                    }
                    *unhandled.get_mut(&dxf_type_name).unwrap() += 1;
                }
            }
        }
        info!(
            "loaded dxf layer: '{}' with {} entities",
            layer.name,
            layer.num_entities()
        );
    }

    for (dxf_type_name, count) in unhandled.iter() {
        warn!("UNHANDLED: {} : {} occurences", dxf_type_name, count);
    }

    layers
}

//makes an svg from a table of objects
fn write_layers_to_svg(layers: &HashMap<String, Layer>, output_path: String) {
    //
    let mut layer_polylines = HashMap::<String, Vec<PolyLine>>::default();
    for (name, layer) in layers.iter() {
        layer_polylines.insert(name.clone(), layer.into_polylines());
    }

    let all_polylines: Vec<PolyLine> = layer_polylines
        .values()
        .flat_map(|v| v.iter().cloned())
        .collect();

    // compute stats for polylines
    let x_values: Vec<f64> = all_polylines
        .iter()
        .flat_map(|e| e.x_values.clone())
        .collect();

    let y_values: Vec<f64> = all_polylines
        .iter()
        .flat_map(|e| e.y_values.clone())
        .collect();

    let cmp = |a: &f64, b: &f64| f64::partial_cmp(a, b).unwrap();
    let min_x = x_values.iter().copied().min_by(cmp).unwrap();
    let max_x = x_values.iter().copied().max_by(cmp).unwrap();
    let min_y = y_values.iter().copied().min_by(cmp).unwrap();
    let max_y = y_values.iter().copied().max_by(cmp).unwrap();

    // create document
    let width = max_x - min_x;
    let height = max_y - min_y;
    let mut document = svg::Document::new()
        // .set::<_, (f64, f64, f64, f64)>("viewBox", (22000.0, 90000.0, 2800.0, 4000.0))
        .set::<_, (f64, f64, f64, f64)>("viewBox", (0.0, 0.0, width, height))
        .set(
            "xmlns:inkscape",
            "http://www.inkscape.org/namespaces/inkscape",
        )
        .set("inkscape:version", "1.1.1 (3bf5ae0d25, 2021-09-20)");

    // insert polylines into svg paths
    for (layer_name, polylines) in layer_polylines.iter() {
        let mut group = svg_element::Group::new()
            .set("inkscape:label", layer_name.as_str())
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

            let stroke: &str = match polyline.stroke.len() {
                0 => "black",
                _ => polyline.stroke.as_str(),
            };

            if polyline.is_closed {
                path_data = path_data.close();
            }

            let path = svg_element::Path::new()
                .set("fill", "none")
                .set("stroke", stroke)
                .set("stroke-width", "10px")
                .set("d", path_data);

            group = group.add(path);
        }

        document = document.add(group);

        info!("created svg layer: {}", layer_name);
    }

    // write to file
    match svg::save(&output_path, &document) {
        Ok(_) => info!("Created file: {}", output_path),
        Err(err) => panic!("Error: {}", err),
    };
}

fn make_polyline_circle(num_segments: usize, c: &dxfe::Circle) -> PolyLine {
    _make_polyline_ellipse(
        num_segments,
        c.center.x,
        c.center.y,
        c.radius,
        0.0,
        1.0,
        c.normal.z,
        0.0,
        2.0 * PI,
    )
}

fn make_polyline_arc(num_segments: usize, a: &dxfe::Arc) -> PolyLine {
    _make_polyline_ellipse(
        num_segments,
        a.center.x,
        a.center.y,
        a.radius,
        0.0,
        1.0,
        a.normal.z,
        a.start_angle * PI / 180.0,
        a.end_angle * PI / 180.0,
    )
}

fn make_polyline_ellipse(num_segments: usize, e: &dxfe::Ellipse) -> PolyLine {
    _make_polyline_ellipse(
        num_segments,
        e.center.x,
        e.center.y,
        e.major_axis.x,
        e.major_axis.y,
        e.minor_axis_ratio,
        e.normal.z,
        e.start_parameter,
        e.end_parameter,
    )
}

/// Can be used to create, circles, arcs and ellipses.
///
/// This works from the fact that circle, arcs, and ellipses are all the special cases of the
/// same generic thing, the generic ellipse.
///
/// An ellipse is defined by it's major and minor axies, see wikipedia.
/// Given a major axis starting at (cx, cy) and ending at (mx, my), the minor axis
/// is known to be 90 degrees to this, and be scaled by a `ratio`. We use this fact
/// and together with some basic trigonometry to compute the ellipse.
fn _make_polyline_ellipse(
    num_segments: usize,
    cx: f64,
    cy: f64,
    mx: f64,
    my: f64,
    ratio: f64,
    normal_z: f64,
    mut a1: f64,
    a2: f64,
) -> PolyLine {
    // this ensures that `da` has correct magnitude and sign ... subtle.
    if a1 > a2 {
        a1 -= 2.0 * PI;
    }

    assert!(num_segments > 0);
    assert!(a1 != a2);
    let da = (a2 - a1) / (num_segments as f64);

    let rx = (mx.powi(2) + my.powi(2)).sqrt();
    let ry = ratio * rx;

    assert!(rx > 0.0);
    // used to rotate the ellipse so that it aligns with major axis vector.
    let cos_rot = mx / rx;
    let sin_rot = my / rx;

    let mut x_values = Vec::<f64>::default();
    let mut y_values = Vec::<f64>::default();

    for n in 0..(num_segments + 1) {
        let a = a1 + (n as f64) * da;

        // ellipse allgined with x-axis as major axis
        let xa = rx * a.cos();
        let ya = ry * a.sin();

        // rotation to align with the real major axis, note sign change when normal_z changes
        let rxa = xa * cos_rot - normal_z.signum() * ya * sin_rot;
        let rya = xa * sin_rot + normal_z.signum() * ya * cos_rot;

        x_values.push(cx + rxa);
        y_values.push(cy + rya);
    }

    let is_closed = ((a2 - a1).abs() - 2.0 * PI).abs() <= 1e-10;

    // because when closed we do not want to include the final point, that would be stuttering.
    if is_closed {
        x_values.pop();
        y_values.pop();
    }

    PolyLine {
        x_values,
        y_values,
        stroke: "red".into(),
        is_closed,
    }
}

