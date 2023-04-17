const NUM_SEGMENTS: usize = 16;
use dxf::{entities::{self as dxfe, Spline, LwPolyline}, LwPolylineVertex, Point};
use serde::{Serialize, Deserialize};
use std::{collections::HashMap, f64::consts::PI, fmt};
use log::{error, info, warn};
use dxfe::EntityType as ET;
//use splines::{Interpolation, Key, Spline as Spline2D};
//use dxf::Drawing;
#[derive(Clone, Default, PartialEq, Serialize, Deserialize, Debug)]
pub struct PolyLine {
    pub is_closed: bool,
    pub x_values: Vec<f64>,
    pub y_values: Vec<f64>,
}
impl PolyLine {
    pub fn new(is_closed: bool, x_values: Vec<f64>, y_values: Vec<f64>) -> Self {
        Self {is_closed, x_values, y_values}
    }
}
impl fmt::Display for PolyLine {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match fmt.write_str(&self.x_values.len().to_string()) {
            Ok(_) => info!(""),
            Err(err) => error!("Error: {}", err)
        };
        Ok(())
    }
}
impl From<dxfe::Line> for PolyLine {
    fn from(e: dxfe::Line) -> Self {
        Self {
            x_values: vec![e.p1.x, e.p2.x],
            y_values: vec![e.p1.y, e.p2.y],
            is_closed: false,
        }
    }
}

impl From<dxfe::LwPolyline> for PolyLine {
    fn from(e: dxfe::LwPolyline) -> Self {
        Self {
            x_values: e.vertices.iter().map(|v| v.x).collect(),
            y_values: e.vertices.iter().map(|v| v.y).collect(),
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
#[derive(Clone, Debug, Default)]
pub struct Layer {
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
    _splines: Vec<dxfe::Spline>,
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

    pub fn into_polylines(&self) -> Vec<PolyLine> {
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
        is_closed,
    }
}
pub fn extract_layers(dxf_file: &dxf::Drawing) -> HashMap<String, Layer> {
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

            match dxf_type {
                // handled entities
                ET::Line(e) => ld.lines.push(e.clone()),
                ET::LwPolyline(e) => ld.lw_polylines.push(e.clone()),
                ET::Arc(e) => ld.arcs.push(e.clone()),
                ET::Circle(e) => ld.circles.push(e.clone()),
                ET::Ellipse(e) => ld.ellipses.push(e.clone()),
                ET::Spline(e) => ld.lw_polylines.push(spline_to_polyline(e.clone())),

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

fn spline_to_polyline(e: Spline) -> LwPolyline{

    let mut lwpoly = dxfe::LwPolyline::default();
    lwpoly.set_is_closed(e.get_is_closed());
    
    let c_p: Vec<Point> = e.control_points;
    let f_p = e.fit_points;
    let vertix: Vec<LwPolylineVertex>;
    //use fit points if available (higher accuracy on points)
    if f_p.len() > 1{
        vertix = f_p.iter().map(|p| LwPolylineVertex{ x: p.x, y: p.y, ..Default::default() }).collect();
    }
    else{
        vertix = c_p.iter().map(|p| LwPolylineVertex{ x: p.x, y: p.y, ..Default::default() }).collect();
    }
    lwpoly.vertices = vertix;
    lwpoly
    
    
}

//all this became unused, as ezdxf in python does all the job
/*pub fn convert_specific_layers(layers: &HashMap<String, Vec<PolyLine>>, layer_names: &Vec<String>, min_x: &f64, min_y: &f64) -> Drawing{
    let mut out_file = Drawing::new();
    for name in layer_names {
        add_layer_to_file(&mut out_file, &layers.get(name).unwrap(), &name, min_x, min_y)
    }
    out_file
}
fn connect_layers(layers: &HashMap<String, Vec<PolyLine>>, min_x: &f64, min_y: &f64) -> Drawing{
    convert_specific_layers(layers,  &layers.keys().cloned().collect(), min_x, min_y)
}

fn add_layer_to_file(dxf_file: &mut dxf::Drawing, layer: &Vec<PolyLine>, layer_name: &String, min_x: &f64, min_y: &f64){
    //println!("Starting layer {}", layer.name);
    let mut new_layer = dxf::tables::Layer::default();
    new_layer.name = layer_name.clone();
    dxf_file.add_layer(new_layer);
    for polyline in layer{
        add_polyline_to_file(dxf_file, &polyline, min_x, min_y, &layer_name);
    }
}
fn add_layer_to_file_no_extras(dxf_file: &mut dxf::Drawing, layer: &Vec<PolyLine>, layer_name: &String) {
    let mut new_layer = dxf::tables::Layer::default();
    new_layer.name = layer_name.clone();
    dxf_file.add_layer(new_layer);
    for polyline in layer{
        add_polyline_to_file_no_extras(dxf_file, &polyline, &layer_name);
    }
}
fn add_polyline_to_file(dxf_file: &mut dxf::Drawing, polyline: &PolyLine, min_x: &f64, min_y: &f64, layer_name: &String){
    let mut new_polyline = dxf::entities::LwPolyline::default();

    let x_values = polyline.x_values.iter();
    let y_values = polyline.y_values.iter();
    let xy_values = x_values.zip(y_values).map(|(x, y)| (x - min_x, y - min_y));
    let mut counter = 0;
    for(x, y) in xy_values{
        let mut vertex = dxf::LwPolylineVertex::default();
        vertex.x = x;
        vertex.y = y;
        vertex.id = counter;
        counter += 1;
        new_polyline.vertices.push(vertex);
    }
    new_polyline.set_is_closed(polyline.is_closed);
    let mut entity = dxf::entities::Entity::new(dxf::entities::EntityType::LwPolyline(new_polyline));
    let mut common = dxf::entities::EntityCommon::default();
    common.layer = layer_name.clone();
    entity.common = common;
    dxf_file.add_entity(entity);
}
fn add_polyline_to_file_no_extras(dxf_file: &mut dxf::Drawing, polyline: &PolyLine, layer_name: &String){
let mut new_polyline = dxf::entities::LwPolyline::default();

    let x_values = polyline.x_values.iter();
    let y_values = polyline.y_values.iter();
    let xy_values = x_values.zip(y_values).map(|(x, y)| (x.clone(), y.clone()));
    let mut counter = 0;
    for(x, y) in xy_values{
        let mut vertex = dxf::LwPolylineVertex::default();
        vertex.x = x;
        vertex.y = y;
        vertex.id = counter;
        counter += 1;
        new_polyline.vertices.push(vertex);
    }
    new_polyline.set_is_closed(polyline.is_closed);
    let mut entity = dxf::entities::Entity::new(dxf::entities::EntityType::LwPolyline(new_polyline));
    let mut common = dxf::entities::EntityCommon::default();
    common.layer = layer_name.clone();
    entity.common = common;
    dxf_file.add_entity(entity);
}
pub fn clone_dxf(in_file: &Drawing) -> Drawing {
    let mut out_file = Drawing::new();
    for (name, layer) in extract_layers(in_file) {
        add_layer_to_file_no_extras(&mut out_file, &layer.into_polylines(), &name);
    }
    out_file
}*/