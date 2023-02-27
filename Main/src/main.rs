#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

const NUM_SEGMENTS: usize = 16;


use eframe::{egui, glow::{FILL, BLUE}};
use egui_extras::image::FitTo;
use egui::Color32;
use clap::Parser;

use dxf::{entities::{self as dxfe, Line, LwPolyline, Polyline}, Point, Drawing};
use std::{collections::HashMap, f64::consts::PI, hash::Hash};
use svg::node::element as svg_element;
use dxfe::EntityType as ET;
use log::{error, info, warn};
/*use line_intersection::{LineInterval, LineRelation};
use geo::{Coordinate, Line as GeoLine, Point as GeoPoint};*/
#[derive(Clone, Default, PartialEq)]
pub struct PolyLine {
    is_closed: bool,
    x_values: Vec<f64>,
    y_values: Vec<f64>,
}

#[derive(Clone, Copy, PartialEq)]
pub struct CustomPoint {
    x: f64,
    y: f64,
}

impl CustomPoint {
    fn new(x: f64, y: f64) -> Self {
        Self {x, y}
    }
    fn clone(&self) -> Self { 
        *self
    }
}
#[derive(Clone, Copy, PartialEq)]
pub struct PointWithNeighbour {
    point: CustomPoint,
    neighbour: CustomPoint
}
impl PointWithNeighbour {
    fn new(point: CustomPoint, neighbour: CustomPoint) -> Self {
        Self {point, neighbour}
    }
    fn clone(&self) -> Self {
        *self
    }
}
impl PolyLine {
    fn new(is_closed: bool, x_values: Vec<f64>, y_values: Vec<f64>) -> Self {
        Self {is_closed, x_values, y_values}
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
#[cfg(not(target_arch = "wasm32"))]
fn main() {
   // load logger from environment
    env_logger::init_from_env(
        env_logger::Env::new()
            .filter("LOG")
            .write_style("LOG_STYLE")
            ,
    ); 
    
    /*let input_path = "test.dxf".to_string();

    let output_path = input_path.clone().replace('.', "_").replace(' ', "_") + "_export.dxf";
    let output_path_svg = input_path.clone().replace('.', "_").replace(' ', "_") + "_export.svg";

    let in_file = dxf::Drawing::load_file(input_path).expect("expexted valid input file");
    let mut dxf_file = dxf::Drawing::new();
    
    let layers = extract_layers(&in_file);
    connect_layers(&layers, dxf_file, &output_path, &output_path_svg); 
*/

    //EGUI
    let native_options = eframe::NativeOptions::default();
    let app = SvgApp::default();
    match eframe::run_native(
        "dxf janitors",
        native_options,
        Box::new(|_cc| Box::new(app)),
    ){
        Ok(_) => info!("Started App!"),
        Err(err) => panic!("Error while starting app: {}", err),
    };

    
    
}
/*fn alter_dxf(in_file: &Drawing) -> Drawing{
    let mut out_file = dxf::Drawing::new();
    let layers = extract_layers(&in_file);
    //let output_path = input_path.clone().replace('.', "_").replace(' ', "_").replace("\\", "/") + "_export.svg";
    //println!("Path: {}", output_path);
    out_file = connect_layers(&layers, out_file); 
    out_file
}*/
fn convert_specific_layers(layers: &HashMap<String, Vec<PolyLine>>, file: &Drawing, layer_names: &Vec<String>, min_x: &f64, min_y: &f64) -> Drawing{
    let mut out_file = Drawing::new();
    for name in layer_names {
        add_layer_to_file(&mut out_file, &layers.get(name).unwrap(), &name, min_x, min_y)
    }
    out_file
}
fn connect_layers(layers: &HashMap<String, Vec<PolyLine>>, file: &Drawing, min_x: &f64, min_y: &f64) -> Drawing{
    convert_specific_layers(layers, file, &layers.keys().cloned().collect(), min_x, min_y)
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



fn add_layer_to_file(dxf_file: &mut dxf::Drawing, layer: &Vec<PolyLine>, layer_name: &String, min_x: &f64, min_y: &f64){
    //println!("Starting layer {}", layer.name);
    let mut new_layer = dxf::tables::Layer::default();
    new_layer.name = layer_name.clone();
    dxf_file.add_layer(new_layer);
    for polyline in layer{
        add_polyline_to_file(dxf_file, &polyline, min_x, min_y, &layer_name);
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

fn extend_closest_lines(in_map: &HashMap<String, Vec<PolyLine>>) -> HashMap<String, Vec<PolyLine>>{
    let mut out_map = HashMap::<String, Vec<PolyLine>>::default();
    for (name, in_polylines) in in_map{
        let mut out_polylines = Vec::<PolyLine>::default();
        let mut xy_ends: Vec<PointWithNeighbour> = Vec::new();
        let mut buddy_map = HashMap::<PointWithNeighbour, PolyLine>::default();
        for polyline in in_polylines {
            if polyline.is_closed {
                out_polylines.push(polyline.clone());
            }

            let len: i32 = polyline.x_values.len().try_into().unwrap();
            let mut x_val = polyline.x_values.clone();
            let mut y_val = polyline.y_values.clone();
            if len == 2 {
                let point = PointWithNeighbour::new(CustomPoint::new(match polyline.x_values.first() {
                    None => 0.0,
                    Some(x) => x.clone(),
                }, match polyline.y_values.first() {
                    None => 0.0,
                    Some(x) => x.clone(),
                }), CustomPoint::new(match polyline.x_values.last() {
                    None => 0.0,
                    Some(x) => x.clone(),
                }, match polyline.y_values.last() {
                    None => 0.0,
                    Some(x) => x.clone(),
                }));
                //buddy_map.insert(point, polyline);
                xy_ends.push(point);
                let point_2 = PointWithNeighbour::new(CustomPoint::new(match polyline.x_values.last() {
                    None => 0.0,
                    Some(x) => x.clone(),
                }, match polyline.y_values.last() {
                    None => 0.0,
                    Some(x) => x.clone(),
                }), CustomPoint::new(match polyline.x_values.first() {
                    None => 0.0,
                    Some(x) => x.clone(),
                }, match polyline.y_values.first() {
                    None => 0.0,
                    Some(x) => x.clone(),
                }));
                xy_ends.push(point_2);
            }
            else if len == 3 {
                let x1 = match x_val.pop(){
                    None => 0.0,
                    Some(x) => x,
                };
                let x2 = match x_val.pop(){
                    None => 0.0,
                    Some(x) => x,
                };
                let x3 = match x_val.pop(){
                    None => 0.0,
                    Some(x) => x,
                };
                let y1 = match y_val.pop(){
                    None => 0.0,
                    Some(x) => x,
                };
                let y2 = match y_val.pop(){
                    None => 0.0,
                    Some(x) => x,
                };
                let y3 = match y_val.pop(){
                    None => 0.0,
                    Some(x) => x,
                };
                let mid_point = CustomPoint::new(x2, y2);
                xy_ends.push(PointWithNeighbour::new(CustomPoint::new(x1, y1), mid_point.clone()));
                xy_ends.push(PointWithNeighbour::new(CustomPoint::new(x3, y3), mid_point.clone()));
            }
            else{
                //Adds the last coordinates to the vector
                xy_ends.push(PointWithNeighbour::new(CustomPoint::new(x_val.pop().unwrap(), y_val.pop().unwrap()), CustomPoint::new(x_val.pop().unwrap(), y_val.pop().unwrap())));
                let mut i = len - 4;
                while i > 0 {
                    x_val.pop();
                    y_val.pop();
                    i -= 1;
                }
                let x1 = match x_val.pop(){
                    None => 0.0,
                    Some(x) => x,
                };
                let y1 = match y_val.pop(){
                    None => 0.0,
                    Some(x) => x,
                };
                xy_ends.push(PointWithNeighbour::new(CustomPoint::new(match x_val.pop(){
                    None => 0.0,
                    Some(x) => x,
                }, match y_val.pop(){
                    None => 0.0,
                    Some(x) => x,
                }), CustomPoint::new(x1, y1)));
            }

        }
        for point in xy_ends.clone(){
            let closest = find_closest_point(point.point, &xy_ends);
            let closest_to_closest = find_closest_point(closest.point, &xy_ends);
            if !point.eq(closest_to_closest) {
                //out_polylines.push
                continue;
            }
            let common_point = connect_points(&point.point, &point.neighbour, &closest.point, &closest.neighbour);
            //out_polylines.push(PolyLine::new(false, ));
        }
        out_map.insert(name.clone(), out_polylines);
    }
    out_map
}

fn connect_points(a1: &CustomPoint, a2: &CustomPoint, b1: &CustomPoint, b2: &CustomPoint) -> CustomPoint{
    //y = mx + b
    let a_m = a2.y - a1.y / a2.x - a1.x;
    let a_b = a1.y - a_m * a1.x;
    let b_m = b2.y - b1.y / b2.x - b1.x;
    let b_b = b1.y - b_m * b1.x;

    //a_m * x + a_b = b_m * x + b_b

    let x = (b_b - a_b) / (a_m - b_m);
    let y = a_m * x + a_b;
    let new_point = CustomPoint::new(x, y);
    new_point
}
fn find_closest_point(point: CustomPoint, vector: &Vec<PointWithNeighbour>) -> &PointWithNeighbour{
    let mut closest_point = vector.first().to_owned().unwrap();
    let mut closest_distance = f64::sqrt((closest_point.point.x - point.x) * (closest_point.point.x - point.x) + (closest_point.point.y - point.y) * (closest_point.point.y - point.y));
    for v_point in vector{
        let new_distance = f64::sqrt((v_point.point.x - point.x) * (v_point.point.x - point.x) + (v_point.point.y - point.y) * (v_point.point.y - point.y));
        if new_distance == 0. {
            continue;
        }
        if new_distance < closest_distance || closest_distance == 0. {
            closest_distance = new_distance;
            closest_point = v_point;
        }
    }
    closest_point
}

//angle at the point two linear functions intercept
//angle for two linear lines: angle = tan^-1 (|m2-m1|/(1+m1m2)) Where m1 is the slope of function A and m2 is the slope of function B
fn angle_between_lines(m1: f64, m2: f64) -> f64{
    let angle = ((m2-m1).abs()/(1.0+m1*m2)).atan();
    angle * 180. / PI
}

//angle between vectors 
//angle = arccos((a*b)/|a||b|) -> where a*b is the dot product and |a| and |b| is the length of the vectors
fn angle_vectors(v1: (f64, f64), v2: (f64, f64)) -> f64{
    //in a tuple the values are v1.0 and v1.1
    let length_v1 = ((v1.0 * v1.0) + (v1.1*v1.1)).sqrt(); //the length of a vector is |u| = sqrt(x^2+y^2)
    let length_v2 = ((v2.0*v2.0) + (v2.1*v2.1)).sqrt();

    let dotproduct = ((v1.0*v2.0) + (v1.1*v2.1)); //dot product of a 2D vector u*v = x1x2 + y1y2

    let angle = (dotproduct/(length_v1 * length_v2)).acos();
    angle * 180. / PI //return angle in degrees (f64)
}

//B is the vertex where the angle is calculated
//function creates two vectors and uses the function angle vectors to return the angle
fn angle_three_points(A: (f64, f64), B: (f64, f64), C: (f64, f64)) -> f64{
    //creating vectors: AB and BC
    let AB = (B.0 - A.0, B.1 - A.1); //vector AB = (B1 - A1, B2 - A2)
    let BC = (C.0 - B.0, C.1 - B.1);

    let angle = angle_vectors(AB, BC);
    angle //return angle
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

pub struct SvgApp {
    svg_image: egui_extras::RetainedImage,
    picked_path: Option<String>,
    loaded_dxf: Drawing,
    current_dxf: Drawing,
    previous_dxf: Vec<Drawing>,
    min_x: f64,
    min_y: f64,
    max_y: f64,
    width: f64,
    height: f64,
    current_svg: svg::Document,
    loaded_layers: HashMap<String, Vec<PolyLine>>,
    current_layers: HashMap<String, Vec<PolyLine>>,
    checkbox_for_layer: HashMap<String, bool>,
    toggled: bool,
    last_toggled: bool,
}

impl Default for SvgApp {
    fn default() -> Self {
        Self {
            svg_image: egui_extras::RetainedImage::from_svg_bytes_with_size(
                "../tmp_file.svg", //path of svg file to display
                include_bytes!("../tmp_file.svg"), 
                FitTo::Size(3840, 2160), //display resolution (need to check performance effect)
            )
            .unwrap(),
            picked_path: Some("".to_string()),
            loaded_dxf: Drawing::new(),
            current_dxf: Drawing::new(),
            previous_dxf: Vec::<Drawing>::new(),
            min_x: 0.0,
            min_y: 0.0,
            max_y: 0.0,
            width: 0.0,
            height: 0.0,
            current_svg: svg::Document::new(),
            loaded_layers: HashMap::<String, Vec<PolyLine>>::default(),
            current_layers: HashMap::<String, Vec<PolyLine>>::default(),
            checkbox_for_layer: HashMap::<String, bool>::default(),
            toggled: true,
            last_toggled: true,
        }
    }
    
}



//start of app
impl eframe::App for SvgApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        //design the frame
        let _my_frame = egui::containers::Frame {
            inner_margin: egui::style::Margin { left: 0.0, right: 0.0, top: 0.0, bottom: 0.0 }, //margins (affects the color-border)
            outer_margin: egui::style::Margin { left: 0.0, right: 0.0, top: 0.0, bottom: 0.0 },
            rounding: egui::Rounding { nw: 1.0, ne: 1.0, sw: 1.0, se: 1.0 },
            shadow: eframe::epaint::Shadow { extrusion: 1.0, color: Color32::YELLOW },
            fill: Color32::WHITE, //background fill color, affected by the margin
            stroke: egui::Stroke::new(2.0, Color32::GOLD),
        };
        egui::SidePanel::right("right_panel").show(ctx, |ui|{
            ui.heading("Useful tools (Hopefully)");
            ui.set_min_size(ui.available_size());
            //ui.checkbox(&mut self.selected, "Test");
            ui.checkbox(&mut self.toggled, "Toggle All On/Off");
            ui.separator();
            let mut checkboxes = HashMap::<String, bool>::default();
            for layer_name in self.loaded_layers.keys() {
                let mut checkval = self.checkbox_for_layer.get(layer_name).unwrap().clone();
                ui.checkbox(&mut checkval, layer_name);
                checkboxes.insert(layer_name.clone(), checkval);
            }
            self.checkbox_for_layer = checkboxes;
            
            if self.toggled != self.last_toggled {
                let mut checkboxes = HashMap::<String, bool>::default();
                for layer_name in self.loaded_layers.keys() {
                    checkboxes.insert(layer_name.clone(), self.toggled);
                }
                self.checkbox_for_layer = checkboxes;
            }
            self.last_toggled = self.toggled;
            if ui.button("Rebuild svg").clicked() {
                let mut temp = HashMap::<String, Vec<PolyLine>>::default();
                for (name, checked) in &self.checkbox_for_layer {
                    if checked.clone(){
                        temp.insert(name.clone(), self.loaded_layers.get(name).unwrap().clone());
                    }
                }
                self.current_layers = temp;
                self.current_dxf = convert_specific_layers(&self.current_layers, &self.loaded_dxf, &self.current_layers.keys().cloned().collect(), &self.min_x, &self.min_y);
                self.current_svg = create_svg(&self.current_layers, &self.min_x, &self.max_y, &self.width, &self.height);
                self.svg_image = egui_extras::RetainedImage::from_svg_bytes_with_size(
                    "test", //path of svg file to display
                    self.current_svg.to_string().as_bytes(), 
                    FitTo::Size(3840, 2160), //display resolution (need to check performance effect)
                )
                .unwrap();
            }
            
        });
        egui::TopBottomPanel::top("bottom_panel").show(ctx, |ui|{
            ui.heading("File Selector");
            //ui.set_min_size(ui.available_size());
            if ui.button("Open file…").clicked() {
                if let Some(path) = rfd::FileDialog::new().pick_file() {
                    self.picked_path = Some(path.display().to_string());
                }
            }
            
            if let Some(picked_path) = &self.picked_path {
                ui.horizontal(|ui| {
                    ui.label("Picked file:");
                    ui.monospace(picked_path);
                });
            }
            if ui.button("Load file!").clicked() {
                //self.selected = true;
                self.loaded_dxf = dxf::Drawing::load_file(self.picked_path.clone().unwrap()).expect("expexted valid input file");
                let mut layer_polylines = HashMap::<String, Vec<PolyLine>>::default();
                let mut layers = extract_layers(&self.loaded_dxf);
                let mut checkbox_map = HashMap::<String, bool>::default();
                
                
                for (name, layer) in layers.iter() {
                    layer_polylines.insert(name.clone(), layer.into_polylines());
                    //layer_color.insert(name.clone(), colors.pop().unwrap().to_owned());
                }
                self.loaded_layers = layer_polylines.clone();
                self.current_layers = layer_polylines.clone();

                for layer_name in self.loaded_layers.keys() {
                    //println!("{}", layer_name);
                    checkbox_map.insert(layer_name.clone(), true);
                }
                self.checkbox_for_layer = checkbox_map;

                let result = calculate_min_max(&layer_polylines);
                self.min_x = result.0;
                self.min_y = result.1;
                self.max_y = result.2;
                self.width = result.3;
                self.height = result.4;

                //self.current_dxf = alter_dxf(&self.loaded_dxf);
                //layers = extract_layers(&self.current_dxf);
                //Colors to use when creating svg.. The last one is used first
                //let mut colors = vec!["%23000000", "%23FF0000", "%23FFFF00", "%2300FF00", "%23008000", "%2300FFFF", "%23008080", "%230000FF", "%23FF00FF", "%23800080", "%23FFA500", "%23FFD700", "%238B4513"];
                
                self.current_svg = create_svg(&layer_polylines, &self.min_x, &self.max_y, &self.width, &self.height);
                self.svg_image = egui_extras::RetainedImage::from_svg_bytes_with_size(
                    "test", //path of svg file to display
                    self.current_svg.to_string().as_bytes(), 
                    FitTo::Size(3840, 2160), //display resolution (need to check performance effect)
                )
                .unwrap();
            }
        });
        //ui the last panel added. this one should only contain our svg if we decide to use multiple panels down the line
        egui::CentralPanel::default().show(ctx, |ui| {
            /*let mut size = ui.available_size();
            size.x = size.x / 1.2;
            size.y = size.y / 1.2;*/
            
            self.svg_image.show_size(ui, ui.available_size());

            
            /*if ui.button("Set!").clicked() {
                ctx.request_repaint();
                println!("Path: {}", self.look_path.clone());
                self.svg_image =  egui_extras::RetainedImage::from_svg_bytes_with_size(
                    self.look_path.clone(), //path of svg file to display
                    self.look_path.as_bytes(), 
                    FitTo::Size(3840, 2160), //display resolution (need to check performance effect)
                )
                .unwrap();
            }*/
        });

    }
} 
fn convert_dxf(file: Drawing) -> String {
    String::new()
}
fn calculate_min_max(layer_polylines: &HashMap<String, Vec<PolyLine>>) -> (f64, f64, f64, f64, f64){
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
    (min_x, min_y, max_y, width, height)
}
/*fn layers_as_svg() -> &'static [u8] {
    let mut document = svg::Document::new()
    // .set::<_, (f64, f64, f64, f64)>("viewBox", (22000.0, 90000.0, 2800.0, 4000.0))
    .set::<_, (f64, f64, f64, f64)>("viewBox", (0., 0., 0., 0.))
    .set(
        "xmlns:inkscape",
        "http://www.inkscape.org/namespaces/inkscape",
    )
    .set("inkscape:version", "1.1.1 (3bf5ae0d25, 2021-09-20)");
document.to_string().as_bytes()
}*/
fn create_svg(layer_polylines: &HashMap<String, Vec<PolyLine>>, min_x: &f64, max_y: &f64, width: &f64, height: &f64) -> svg::Document{
    let mut document = svg::Document::new()
    // .set::<_, (f64, f64, f64, f64)>("viewBox", (22000.0, 90000.0, 2800.0, 4000.0))
        .set::<_, (f64, f64, f64, f64)>("viewBox", (0.0, 0.0, width.clone(), height.clone()))
        .set(
        "xmlns:inkscape",
        "http://www.inkscape.org/namespaces/inkscape",
    )
        .set("inkscape:version", "1.1.1 (3bf5ae0d25, 2021-09-20)");


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
                .set("stroke-width", "0.03px")
                .set("d", path_data);

            group = group.add(path);
        }

        document = document.add(group);

        info!("created svg layer: {}", name);
    }
    document
}
