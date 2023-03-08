#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod dxfwrite;
mod algorithms;
use algorithms::{CustomPoint, PointWithNeighbour};
mod svgwrite;
mod dxfextract;
use dxfextract::PolyLine;
use eframe::{egui, glow::{FILL, BLUE}};
use egui_extras::image::FitTo;
use egui::Color32;
//use clap::Parser;

use pyo3::prelude::*;
//use dxf::{entities::{self as dxfe, Line, LwPolyline, Polyline}, Point, Drawing};
use dxf::Drawing;
use svg::Document;
use std::{collections::HashMap, f64::consts::PI, hash::Hash};
use log::{error, info, warn};
/*use line_intersection::{LineInterval, LineRelation};
use geo::{Coordinate, Line as GeoLine, Point as GeoPoint};*/


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

pub struct SvgApp {
    svg_image: egui_extras::RetainedImage,
    picked_path: Option<String>,
    loaded_dxf: Drawing,
    current_dxf: Drawing,
    previous_dxfs: Vec<Drawing>,
    next_dxfs: Vec<Drawing>,
    previous_svgs: Vec<svg::Document>,
    next_svgs: Vec<svg::Document>,
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
            previous_dxfs: Vec::<Drawing>::new(),
            next_dxfs: Vec::<Drawing>::new(),
            previous_svgs: Vec::<Document>::new(),
            next_svgs: Vec::<Document>::new(),
            min_x: 0.0,
            min_y: 0.0,
            max_y: 0.0,
            width: 0.0,
            height: 0.0,
            current_svg: Document::new(),
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
            ui.horizontal(|ui|{
                if ui.button("Undo").clicked() {
                    self.next_dxfs.push(dxfextract::clone_dxf(&self.current_dxf));
                    self.next_svgs.push(self.current_svg.clone());
                    self.current_dxf = match self.previous_dxfs.pop(){
                        None => dxfextract::clone_dxf(&self.current_dxf),
                        Some(x) => x,
                    };
                    self.current_svg = match self.previous_svgs.pop(){
                        None => self.current_svg.clone(),
                        Some(x) => x,
                    };
                    self.svg_image = egui_extras::RetainedImage::from_svg_bytes_with_size(
                        "test", //path of svg file to display
                        self.current_svg.to_string().as_bytes(), 
                        FitTo::Size(3840, 2160), //display resolution (need to check performance effect)
                    )
                    .unwrap();
                }
                if ui.button("Redo").clicked() {
                    self.previous_dxfs.push(dxfextract::clone_dxf(&self.current_dxf));
                    self.previous_svgs.push(self.current_svg.clone());
                    self.current_dxf = match self.next_dxfs.pop(){
                        None => dxfextract::clone_dxf(&self.current_dxf),
                        Some(x) => x,
                    };
                    self.current_svg = match self.next_svgs.pop(){
                        None => self.current_svg.clone(),
                        Some(x) => x,
                    };
                    self.svg_image = egui_extras::RetainedImage::from_svg_bytes_with_size(
                        "test", //path of svg file to display
                        self.current_svg.to_string().as_bytes(), 
                        FitTo::Size(3840, 2160), //display resolution (need to check performance effect)
                    )
                    .unwrap();
                }
            });
            
            ui.separator();
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
                self.previous_dxfs.push(dxfextract::clone_dxf(&self.current_dxf));
                self.previous_svgs.push(self.current_svg.clone());
                //TODO fix a better way to store previous files, so we can remove the first of them after a certain treshold
                //println!("Length of DXF-vector: {}", self.previous_dxfs.len());
                self.current_dxf = dxfextract::convert_specific_layers(&self.current_layers, &self.current_layers.keys().cloned().collect(), &self.min_x, &self.min_y);
                self.current_svg = svgwrite::create_svg(&self.current_layers, &self.min_x, &self.max_y, &self.width, &self.height);
                self.svg_image = egui_extras::RetainedImage::from_svg_bytes_with_size(
                    "test", //path of svg file to display
                    self.current_svg.to_string().as_bytes(), 
                    FitTo::Size(3840, 2160), //display resolution (need to check performance effect)
                )
                .unwrap();
            }
            
        });
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui|{
            ui.horizontal(|ui|{
                ui.heading("File Selector");
                //ui.set_min_size(ui.available_size());
                if ui.button("Open file…").clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_file() {
                        self.picked_path = Some(path.display().to_string());
                    }
                }
                if ui.button("Save original as SVG (ezdxf)").clicked() {
                    if !&self.picked_path.clone().unwrap().eq("") {
                        match svgwrite::save_svg_ez(&self.picked_path.clone().unwrap()){
                            Ok(_) => info!("DXF saved!"),
                            Err(err) => panic!("Error while saving DXF: {}", err),
                        };
                    }
                    
                }
                if ui.button("Save as SVG").clicked() {
                    if !&self.picked_path.clone().unwrap().eq("") {
                        svgwrite::save_svg(&self.picked_path.clone().unwrap(), &self.current_svg);
                    }
                    
                }
                if ui.button("Save as DXF (WIP)").clicked() {
                    if !&self.picked_path.clone().unwrap().eq("") {
                        match dxfwrite::savedxf(self.current_layers.clone(), &self.picked_path.clone().unwrap()){
                            Ok(_) => info!("DXF saved!"),
                            Err(err) => panic!("Error while saving DXF: {}", err),
                        };
                        //dxfwrite::savedxf(self.current_layers.clone(), &self.picked_path.clone().unwrap());
                    }
                    
                }
                
                
            });
            
            if let Some(picked_path) = &self.picked_path {
                ui.horizontal(|ui| {
                    ui.label("Picked file:");
                    ui.monospace(picked_path);
                });
            }
            
            if ui.button("Load file!").clicked() {
                //self.selected = true;
                self.previous_dxfs = Vec::<Drawing>::new();
                self.next_dxfs = Vec::<Drawing>::new();
                self.previous_svgs = Vec::<svg::Document>::new();
                self.next_svgs = Vec::<svg::Document>::new();
                self.loaded_dxf = dxf::Drawing::load_file(self.picked_path.clone().unwrap()).expect("expexted valid input file");
                let mut layer_polylines = HashMap::<String, Vec<PolyLine>>::default();
                let layers = dxfextract::extract_layers(&self.loaded_dxf);
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

                let result = algorithms::calculate_min_max(&layer_polylines);
                self.min_x = result.0;
                self.min_y = result.1;
                self.max_y = result.2;
                self.width = result.3;
                self.height = result.4;

                //self.current_dxf = alter_dxf(&self.loaded_dxf);
                //layers = extract_layers(&self.current_dxf);
                //Colors to use when creating svg.. The last one is used first
                //let mut colors = vec!["%23000000", "%23FF0000", "%23FFFF00", "%2300FF00", "%23008000", "%2300FFFF", "%23008080", "%230000FF", "%23FF00FF", "%23800080", "%23FFA500", "%23FFD700", "%238B4513"];
                
                self.current_svg = svgwrite::create_svg(&layer_polylines, &self.min_x, &self.max_y, &self.width, &self.height);
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


