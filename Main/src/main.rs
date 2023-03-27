#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod dxfwrite;
mod algorithms;
use algorithms::{CustomPoint, PointWithNeighbour};
mod svgwrite;
mod dxfextract;
use dxfextract::PolyLine;
use eframe::{egui, glow::{FILL, BLUE}, epaint::ahash::HashSet};
use egui_extras::image::FitTo;
use egui::{Color32, ScrollArea, Ui, text, FontDefinitions, Button, Align2};
//use clap::Parser;

use pyo3::prelude::*;
//use dxf::{entities::{self as dxfe, Line, LwPolyline, Polyline}, Point, Drawing};
use dxf::Drawing;
use svg::Document;
use std::{collections::HashMap, f64::consts::PI, hash::Hash, ffi::OsStr, /*default::default*/};
use log::{error, info, warn};
use egui::{Sense, Slider, Vec2};

/*use line_intersection::{LineInterval, LineRelation};
use geo::{Coordinate, Line as GeoLine, Point as GeoPoint};*/
enum UndoType {
    Loaded,
    Current,
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

pub struct SvgApp {
    svg_image: egui_extras::RetainedImage,
    picked_path: Option<String>,
    loaded_dxf: Drawing,
    //current_dxf: Drawing,
    //previous_dxfs: Vec<Drawing>,
    //next_dxfs: Vec<Drawing>,
    //previous_svgs: Vec<svg::Document>,
    //next_svgs: Vec<svg::Document>,
    min_x: f64,
    min_y: f64,
    max_y: f64,
    width: f64,
    height: f64,
    current_svg: svg::Document,

    loaded_layers: HashMap<String, Vec<PolyLine>>,
    //Handles the undo system
    undo_stack: Vec<UndoType>,
    redo_stack: Vec<UndoType>,
    prev_l_layers: Vec<HashMap<String, Vec<PolyLine>>>,
    next_l_layers: Vec<HashMap<String, Vec<PolyLine>>>,
    current_layers: HashMap<String, Vec<PolyLine>>,
    prev_c_layers: Vec<HashMap<String, Vec<PolyLine>>>,
    next_c_layers: Vec<HashMap<String, Vec<PolyLine>>>,
    checkbox_for_layer: HashMap<String, bool>,
    old_to_new_name: HashMap::<String, String>,
    toggled: bool,
    last_toggled: bool,
    iterations_slider_value: i32,
    max_angle_slider_value: i32,
    max_distance_slider_value: i32,
    merge_name: String,
    pressed_keys: HashSet<egui::Key>,
    //SLIDERS
    //min: f64,
    //max: f64,
    //step: f64,
    //use_steps: bool,
    //integer: bool,
    //vertical: bool,
    //value: f64,
    //trailing_fill: bool,
    //scalar: f32,
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
            //current_dxf: Drawing::new(),
            //previous_dxfs: Vec::<Drawing>::new(),
            //next_dxfs: Vec::<Drawing>::new(),
            //previous_svgs: Vec::<Document>::new(),
            //next_svgs: Vec::<Document>::new(),
            min_x: 0.0,
            min_y: 0.0,
            max_y: 0.0,
            width: 0.0,
            height: 0.0,
            current_svg: Document::new(),
            loaded_layers: HashMap::<String, Vec<PolyLine>>::default(),
            undo_stack: Vec::<UndoType>::default(),
            redo_stack: Vec::<UndoType>::default(),
            prev_l_layers: Vec::<HashMap<String, Vec<PolyLine>>>::default(),
            next_l_layers: Vec::<HashMap<String, Vec<PolyLine>>>::default(),
            current_layers: HashMap::<String, Vec<PolyLine>>::default(),
            prev_c_layers: Vec::<HashMap<String, Vec<PolyLine>>>::default(),
            next_c_layers: Vec::<HashMap<String, Vec<PolyLine>>>::default(),
            checkbox_for_layer: HashMap::<String, bool>::default(),
            old_to_new_name: HashMap::<String, String>::default(),
            toggled: true,
            last_toggled: true,
            iterations_slider_value: 1,
            max_angle_slider_value: 360,
            max_distance_slider_value: 100,
            merge_name: String::new(),
            pressed_keys: HashSet::<egui::Key>::default(),
            //SLIDERS
            //min: (0.0), 
            //max: (100.0), 
            //step: (1.0), 
            //use_steps: (false), 
            //integer: (false), 
            //vertical: (false), 
            //value: (10.0), 
            //trailing_fill: (true),
            //scalar:(10.0)  
        }
    }
    
}



//start of app
impl eframe::App for SvgApp {
    fn update(&mut self, ctx: &egui::Context, _my_frame: &mut eframe::Frame) {
        //design the frame
        let _my_frame = egui::containers::Frame {
            inner_margin: egui::style::Margin { left: 10.0, right: 10.0, top: 10.0, bottom: 10.0 }, //margins (affects the color-border)
            outer_margin: egui::style::Margin { left: 10.0, right: 10.0, top: 10.0, bottom: 10.0 },
            rounding: egui::Rounding { nw: 1.0, ne: 1.0, sw: 1.0, se: 1.0 },
            shadow: eframe::epaint::Shadow { extrusion: 1.0, color: Color32::YELLOW },
            fill: Color32::from_rgb(23,26,29), //background fill color, affected by the margin
            stroke: egui::Stroke::new(2.0, Color32::BLACK),
        };

        //let mut fonts = FontDefinitions::default();
        //key handler
        ctx.input(|i| {
            //let mut new_set = HashSet::<egui::Key>::default();
            for event in i.events.clone() {
                match event {
                    egui::Event::Key{key, pressed, modifiers: _, repeat: _ } => {
                        //println!("{:?} = {:?}", key, pressed);
                        if pressed {
                            self.pressed_keys.insert(key);
                        }
                        else{
                            if self.pressed_keys.contains(&key){
                                self.pressed_keys.remove(&key);
                            }
                        }
                        
                    },
                    egui::Event::Text(t) => { /*println!("Text = {:?}", t)*/ } _ => {}
                }
            }
            if self.pressed_keys.contains(&egui::Key::ArrowDown) && self.pressed_keys.contains(&egui::Key::N){
                if let Some(path) = rfd::FileDialog::new().add_filter("dxf", &["dxf"]).pick_file() {
                    self.picked_path = Some(path.display().to_string());

                    //get extension to see if we want to update display
                    let extension = path.extension().unwrap();
                    if extension == "dxf"{
                        //self.selected = true;
                        //self.previous_dxfs = Vec::<Drawing>::new();
                        //self.next_dxfs = Vec::<Drawing>::new();
                        //self.previous_svgs = Vec::<svg::Document>::new();
                        //self.next_svgs = Vec::<svg::Document>::new();
                        self.prev_c_layers = Vec::<HashMap<String, Vec<PolyLine>>>::default();
                        self.next_c_layers = Vec::<HashMap<String, Vec<PolyLine>>>::default();
                        self.loaded_dxf = dxf::Drawing::load_file(self.picked_path.clone().unwrap()).expect("Not a valid file");
                        let mut layer_polylines = HashMap::<String, Vec<PolyLine>>::default();
                        let layers = dxfextract::extract_layers(&self.loaded_dxf);
                        let mut checkbox_map = HashMap::<String, bool>::default();
                        let mut old_name_map = HashMap::<String, String>::default();

                        for (name, layer) in layers.iter() {
                            layer_polylines.insert(name.clone(), layer.into_polylines());
                            //layer_color.insert(name.clone(), colors.pop().unwrap().to_owned());
                        }

                        self.loaded_layers = layer_polylines.clone();
                        self.current_layers = layer_polylines.clone();

                        for layer_name in self.loaded_layers.keys() {
                            //println!("{}", layer_name);
                            checkbox_map.insert(layer_name.clone(), true);
                            old_name_map.insert(layer_name.clone(), layer_name.clone());
                        }

                        self.checkbox_for_layer = checkbox_map;
                        self.old_to_new_name = old_name_map;

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
            }
            else if self.pressed_keys.contains(&egui::Key::ArrowDown) && self.pressed_keys.contains(&egui::Key::S){
                if !&self.picked_path.clone().unwrap().eq("") {
                    let res = rfd::FileDialog::new().set_file_name("export").set_directory(&self.picked_path.clone().unwrap()).add_filter("dxf", &["dxf"]).add_filter("svg", &["svg"]).save_file();
                    
                    if let Some(extension) = res{
                        let filetype = extension.extension().unwrap(); //get extension
                        let filepath = extension.as_path().as_os_str().to_os_string().into_string().unwrap(); //convert from &OsStr to String

                        //save dxf
                        if filetype == "dxf"{
                            let mut out_layers = HashMap::<String, Vec<PolyLine>>::default();
                            for (layer_name, polylines) in &self.current_layers{
                                out_layers.insert(self.old_to_new_name.get(layer_name).unwrap().clone(), polylines.clone());
                            }
                            match dxfwrite::savedxf(out_layers, &filepath){
                                Ok(_) => info!("DXF saved!"),
                                Err(err) => panic!("Error while saving DXF: {}", err),
                            };
                        }
                        //save svg
                        else if filetype == "svg"{
                            svgwrite::save_svg(&filepath, &self.current_svg);
                        }
                        //pop-up message error
                        else{
                            let _msg = rfd::MessageDialog::new().set_title("Error!").set_description("Something went wrong while saving. Did you chose the correct extension?").set_buttons(rfd::MessageButtons::Ok).show();
                        
                        }
                    }
                    

                    
                    
                }
            }
            /*for key in &self.pressed_keys {
                println!("{:?}", key);
            }*/
        }
        });
        

        egui::SidePanel::right("right_panel").frame(_my_frame).show(ctx, |ui|{
            ui.heading("Tools:");
            ui.set_min_size(ui.available_size());
            



            //ui.checkbox(&mut self.selected, "Test");
            ui.horizontal(|ui|{
                if ui.button("Undo").clicked() {
                    //self.next_dxfs.push(dxfextract::clone_dxf(&self.current_dxf));
                    //self.next_svgs.push(self.current_svg.clone());
                    if let Some(undo_type) = self.undo_stack.pop() {
                        match undo_type {
                            UndoType::Loaded => {
                                if let Some(prev) = self.prev_l_layers.pop() {
                                    self.undo_stack.push(UndoType::Loaded);
                                    self.next_l_layers.push(self.loaded_layers.clone());
                                    self.loaded_layers = prev;
                                    //let mut out_layers = HashMap::<String, Vec<PolyLine>>::default();
                                    for (name, test) in &self.old_to_new_name {
                                        println!("{} : {}", name, test);
                                    }
                                    /*for (layer_name, polylines) in &self.current_layers{
                                        
                                        println!("Layername: {}", layer_name);
                                        out_layers.insert(self.old_to_new_name.get(layer_name).unwrap().clone(), polylines.clone());
                                    }*/
                                    let mut checkbox_map = HashMap::<String, bool>::default();
                                    let mut old_name_map = HashMap::<String, String>::default();
                                    for layer_name in self.loaded_layers.keys() {
                                        //println!("{}", layer_name);
                                        checkbox_map.insert(layer_name.clone(), true);
                                        old_name_map.insert(layer_name.clone(), layer_name.clone());
                                    }
        
                                    self.checkbox_for_layer = checkbox_map;
                                    self.old_to_new_name = old_name_map;
                                    self.current_svg = svgwrite::create_svg(&self.loaded_layers, &self.min_x, &self.max_y, &self.width, &self.height);
                                    self.svg_image = egui_extras::RetainedImage::from_svg_bytes_with_size(
                                        "test", //path of svg file to display
                                        self.current_svg.to_string().as_bytes(), 
                                        FitTo::Size(3840, 2160), //display resolution (need to check performance effect)
                                    )
                                    .unwrap();
            
                                    /*let mut temp = HashMap::<String, bool>::default();
                                    for (name, _polylines) in &self.loaded_layers {
                                        if self.current_layers.contains_key(name){
                                            temp.insert(name.clone(), true);
                                            continue;
                                        }
                                        temp.insert(name.clone(), false);
                                    }
                                    self.checkbox_for_layer = temp;*/
                                }
                            },
                            UndoType::Current => {
                                if let Some(prev) = self.prev_c_layers.pop() {
                                    self.undo_stack.push(UndoType::Current);
                                    self.next_c_layers.push(self.current_layers.clone());
                                    self.current_layers = prev;
                                    let mut out_layers = HashMap::<String, Vec<PolyLine>>::default();
                                    for (layer_name, polylines) in &self.current_layers{
                                        out_layers.insert(self.old_to_new_name.get(layer_name).unwrap().clone(), polylines.clone());
                                    }
                                    self.current_svg = svgwrite::create_svg(&out_layers, &self.min_x, &self.max_y, &self.width, &self.height);
                                    self.svg_image = egui_extras::RetainedImage::from_svg_bytes_with_size(
                                        "test", //path of svg file to display
                                        self.current_svg.to_string().as_bytes(), 
                                        FitTo::Size(3840, 2160), //display resolution (need to check performance effect)
                                    )
                                    .unwrap();
            
                                    let mut temp = HashMap::<String, bool>::default();
                                    for (name, _polylines) in &self.loaded_layers {
                                        if self.current_layers.contains_key(name){
                                            temp.insert(name.clone(), true);
                                            continue;
                                        }
                                        temp.insert(name.clone(), false);
                                    }
                                    self.checkbox_for_layer = temp;
                                }
                            },
                        };
                    } 
                    
                    
                    /*self.current_dxf = match self.previous_dxfs.pop(){
                        None => dxfextract::clone_dxf(&self.current_dxf),
                        Some(x) => x,
                    };*/
                    /*self.current_svg = match self.previous_svgs.pop(){
                        None => self.current_svg.clone(),
                        Some(x) => x,
                    };*/
                    
                }
                if ui.button("Redo").clicked() {
                    //self.previous_dxfs.push(dxfextract::clone_dxf(&self.current_dxf));
                    //self.previous_svgs.push(self.current_svg.clone());
                    /*self.current_dxf = match self.next_dxfs.pop(){
                        None => dxfextract::clone_dxf(&self.current_dxf),
                        Some(x) => x,
                    };*/
                    /*self.current_svg = match self.next_svgs.pop(){
                        None => self.current_svg.clone(),
                        Some(x) => x,
                    };*/
                    if let Some(undo_type) = self.redo_stack.pop() {
                        match undo_type {
                            UndoType::Loaded => {
                                if let Some(next) = self.next_l_layers.pop(){
                                    self.undo_stack.push(UndoType::Loaded);
                                    self.prev_l_layers.push(self.current_layers.clone());
                                    self.loaded_layers = next;
                                    /*let mut out_layers = HashMap::<String, Vec<PolyLine>>::default();
                                    for (layer_name, polylines) in &self.current_layers{
                                        out_layers.insert(self.old_to_new_name.get(layer_name).unwrap().clone(), polylines.clone());
                                    }*/
                                    self.current_svg = svgwrite::create_svg(&self.loaded_layers, &self.min_x, &self.max_y, &self.width, &self.height);
                                    self.svg_image = egui_extras::RetainedImage::from_svg_bytes_with_size(
                                        "test", //path of svg file to display
                                        self.current_svg.to_string().as_bytes(), 
                                        FitTo::Size(3840, 2160), //display resolution (need to check performance effect)
                                    )
                                    .unwrap();
                                    let mut checkbox_map = HashMap::<String, bool>::default();
                                    let mut old_name_map = HashMap::<String, String>::default();
                                    for layer_name in self.loaded_layers.keys() {
                                        //println!("{}", layer_name);
                                        checkbox_map.insert(layer_name.clone(), true);
                                        old_name_map.insert(layer_name.clone(), layer_name.clone());
                                    }
        
                                    self.checkbox_for_layer = checkbox_map;
                                    self.old_to_new_name = old_name_map;
                                    /*let mut temp = HashMap::<String, bool>::default();
                                    for (name, _polylines) in &self.loaded_layers {
                                        if self.current_layers.contains_key(name){
                                            temp.insert(name.clone(), true);
                                            continue;
                                        }
                                        temp.insert(name.clone(), false);
                                    }
                                    self.checkbox_for_layer = temp;*/
                                }
                            },
                            UndoType::Current => {
                                if let Some(next) = self.next_c_layers.pop(){
                                    self.undo_stack.push(UndoType::Current);
                                    self.prev_c_layers.push(self.current_layers.clone());
                                    self.current_layers = next;
                                    let mut out_layers = HashMap::<String, Vec<PolyLine>>::default();
                                    for (layer_name, polylines) in &self.current_layers{
                                        out_layers.insert(self.old_to_new_name.get(layer_name).unwrap().clone(), polylines.clone());
                                    }
                                    self.current_svg = svgwrite::create_svg(&out_layers, &self.min_x, &self.max_y, &self.width, &self.height);
                                    self.svg_image = egui_extras::RetainedImage::from_svg_bytes_with_size(
                                        "test", //path of svg file to display
                                        self.current_svg.to_string().as_bytes(), 
                                        FitTo::Size(3840, 2160), //display resolution (need to check performance effect)
                                    )
                                    .unwrap();
                                    let mut temp = HashMap::<String, bool>::default();
                                    for (name, _polylines) in &self.loaded_layers {
                                        if self.current_layers.contains_key(name){
                                            temp.insert(name.clone(), true);
                                            continue;
                                        }
                                        temp.insert(name.clone(), false);
                                    }
                                    self.checkbox_for_layer = temp;
                                }
                            },
                        }
                    }
                    
                    
                }
                if ui.button("Connect").clicked(){
                    self.undo_stack.push(UndoType::Current);
                    self.prev_c_layers.push(self.current_layers.clone());
                    let mut temp = HashMap::<String, Vec<PolyLine>>::default();
                    for (name, checked) in &self.checkbox_for_layer {
                        if checked.clone(){
                            temp.insert(name.clone(), self.loaded_layers.get(name).unwrap().clone());
                        }
                    }
                    self.current_layers = algorithms::try_to_close_polylines(false, &self.current_layers, &temp, &Some((self.max_distance_slider_value as f64) / 100. * self.width), &Some(self.max_angle_slider_value), &Some(self.iterations_slider_value));
                    
                    
                    let mut out_layers = HashMap::<String, Vec<PolyLine>>::default();
                    for (layer_name, polylines) in &self.current_layers{
                        out_layers.insert(self.old_to_new_name.get(layer_name).unwrap().clone(), polylines.clone());
                    }
                    self.current_svg = svgwrite::create_svg(&out_layers, &self.min_x, &self.max_y, &self.width, &self.height);
                    self.next_c_layers = Vec::<HashMap<String, Vec<PolyLine>>>::default();
                    self.svg_image = egui_extras::RetainedImage::from_svg_bytes_with_size(
                    "test", //path of svg file to display
                    self.current_svg.to_string().as_bytes(), 
                    FitTo::Size(3840, 2160), //display resolution (need to check performance effect)
                )
                .unwrap();
                }
                if ui.button("Extend").clicked(){
                    self.undo_stack.push(UndoType::Current);
                    self.prev_c_layers.push(self.current_layers.clone());
                    let mut temp = HashMap::<String, Vec<PolyLine>>::default();
                    for (name, checked) in &self.checkbox_for_layer {
                        if checked.clone(){
                            temp.insert(name.clone(), self.loaded_layers.get(name).unwrap().clone());
                        }
                    }
                    self.current_layers = algorithms::try_to_close_polylines(true, &self.current_layers, &temp, &Some((self.max_distance_slider_value as f64) / 100. * self.width), &Some(self.max_angle_slider_value), &Some(self.iterations_slider_value));
                    let mut out_layers = HashMap::<String, Vec<PolyLine>>::default();
                    for (layer_name, polylines) in &self.current_layers{
                        out_layers.insert(self.old_to_new_name.get(layer_name).unwrap().clone(), polylines.clone());
                    }
                    self.current_svg = svgwrite::create_svg(&out_layers, &self.min_x, &self.max_y, &self.width, &self.height);
                    self.next_c_layers = Vec::<HashMap<String, Vec<PolyLine>>>::default();
                    self.svg_image = egui_extras::RetainedImage::from_svg_bytes_with_size(
                    "test", //path of svg file to display
                    self.current_svg.to_string().as_bytes(), 
                    FitTo::Size(3840, 2160), //display resolution (need to check performance effect)
                )
                .unwrap();
                }
                    
            });

            ui.separator();

            // SLIDERS
            

            // wrap the slider in a vertical layout to move it to a new line
            ui.vertical(|ui| {
            //ui.add(egui::Label::new("Iterations"));
                ui.add(Slider::new(&mut self.iterations_slider_value, 1..=10).text("Iterations (amount)"));
                ui.add(Slider::new(&mut self.max_distance_slider_value, 1..=100).text("Max distance (%)"));
                ui.add(Slider::new(&mut self.max_angle_slider_value, 1..=180).text("Max angle (°)"));
            // do not update value with slider_value when slider is change
            });

            // assign the final slider value to value after the UI is drawn

            
            ui.separator();
            ui.checkbox(&mut self.toggled, "Toggle All On/Off");
            ui.separator();
            let mut checkboxes = HashMap::<String, bool>::default();
            let mut new_layer_names = HashMap::<String, String>::default();
            egui::ScrollArea::vertical().show(ui, |ui|{
                for layer_name in self.loaded_layers.keys() {
                    let mut checkval = self.checkbox_for_layer.get(layer_name).unwrap().clone();
                    let mut new_name = self.old_to_new_name.get(layer_name).unwrap().clone();
                    ui.horizontal(|ui|{
                        ui.checkbox(&mut checkval, "");
                        ui.add(egui::TextEdit::singleline(&mut new_name));
                    });
                    checkboxes.insert(layer_name.clone(), checkval);
                    new_layer_names.insert(layer_name.clone(), new_name);
                    ui.separator();
                }
                self.checkbox_for_layer = checkboxes;
                self.old_to_new_name = new_layer_names;
                

                //code for toggle on/off for all layers
                if self.toggled != self.last_toggled {
                    let mut checkboxes = HashMap::<String, bool>::default();
                    for layer_name in self.loaded_layers.keys() {
                        checkboxes.insert(layer_name.clone(), self.toggled);
                    }
                    self.checkbox_for_layer = checkboxes;
                }
                self.last_toggled = self.toggled;
                
            });
            
            self.last_toggled = self.toggled;
            if ui.button("Rebuild svg").clicked() {
                let mut temp = HashMap::<String, Vec<PolyLine>>::default();
                for (name, checked) in &self.checkbox_for_layer {
                    if checked.clone(){
                        temp.insert(name.clone(), self.loaded_layers.get(name).unwrap().clone());
                    }
                }
                self.undo_stack.push(UndoType::Current);
                self.prev_c_layers.push(self.current_layers.clone());
                self.current_layers = temp;
                self.next_c_layers = Vec::<HashMap<String, Vec<PolyLine>>>::default();
                //self.previous_dxfs.push(dxfextract::clone_dxf(&self.current_dxf));
                //self.previous_svgs.push(self.current_svg.clone());
                
                //TODO fix a better way to store previous files, so we can remove the first of them after a certain treshold
                //println!("Length of DXF-vector: {}", self.previous_dxfs.len());
                //self.current_dxf = dxfextract::convert_specific_layers(&self.current_layers, &self.current_layers.keys().cloned().collect(), &self.min_x, &self.min_y);
                let mut out_layers = HashMap::<String, Vec<PolyLine>>::default();
                    for (layer_name, polylines) in &self.current_layers{
                        out_layers.insert(self.old_to_new_name.get(layer_name).unwrap().clone(), polylines.clone());
                    }
                self.current_svg = svgwrite::create_svg(&out_layers, &self.min_x, &self.max_y, &self.width, &self.height);
                self.svg_image = egui_extras::RetainedImage::from_svg_bytes_with_size(
                    "test", //path of svg file to display
                    self.current_svg.to_string().as_bytes(), 
                    FitTo::Size(3840, 2160), //display resolution (need to check performance effect)
                )
                .unwrap();
            }
            ui.horizontal(|ui|{
                if ui.button("Merge").clicked(){
                    //checks wheter the name is in use or not
                    //if let None = self.loaded_layers.get(&self.merge_name){
                        let mut full_layer = Vec::<PolyLine>::default();
                        //let mut out_map = HashMap::<String, Vec<PolyLine>>::default();
                        if self.merge_name == "".to_string() || self.loaded_layers.contains_key(&self.merge_name){
                            let _msg = rfd::MessageDialog::new().set_title("Error!").set_description("The new layer needs different name!").set_buttons(rfd::MessageButtons::Ok).show();
                        }
                        for (layer_name, is_checked) in &self.checkbox_for_layer{
                            if !is_checked {
                                //out_map.insert(layer_name.clone(), self.loaded_layers.get(layer_name).unwrap().clone());
                                continue;
                            }
                            full_layer.append(&mut self.loaded_layers.get(layer_name).unwrap().clone());
                        }
                        self.undo_stack.push(UndoType::Loaded);
                        self.prev_l_layers.push(self.loaded_layers.clone());
                        //out_map.insert(self.merge_name.clone(), full_layer);
                        self.loaded_layers.insert(self.merge_name.clone(), full_layer);
                        self.checkbox_for_layer.insert(self.merge_name.clone(), false);
                        self.old_to_new_name.insert(self.merge_name.clone(), self.merge_name.clone());
                        self.merge_name = String::new();
                        //self.loaded_layers = out_map.clone();

                        //self.prev_layers = Vec::<HashMap<String, Vec<PolyLine>>>::default();
                        //self.next_layers = Vec::<HashMap<String, Vec<PolyLine>>>::default();
                        //let mut layer_polylines = HashMap::<String, Vec<PolyLine>>::default();
                        //let layers = dxfextract::extract_layers(&self.loaded_dxf);
                        //let mut checkbox_map = HashMap::<String, bool>::default();
                        //let mut old_name_map = HashMap::<String, String>::default();
                
                        /*self.current_layers = out_map.clone();

                        for layer_name in self.loaded_layers.keys() {
                            //println!("{}", layer_name);
                            checkbox_map.insert(layer_name.clone(), true);
                            old_name_map.insert(layer_name.clone(), layer_name.clone());
                        }
                        self.checkbox_for_layer = checkbox_map;
                        self.old_to_new_name = old_name_map;
                        self.current_svg = svgwrite::create_svg(&out_map, &self.min_x, &self.max_y, &self.width, &self.height);
                        self.svg_image = egui_extras::RetainedImage::from_svg_bytes_with_size(
                            "test", //path of svg file to display
                            self.current_svg.to_string().as_bytes(), 
                        FitTo::Size(3840, 2160), //display resolution (need to check performance effect)
                        )
                        .unwrap();*/
                    //}
                    
                }
                ui.add(egui::TextEdit::singleline(&mut self.merge_name));
            });
            if ui.button("Delete").clicked(){
                self.undo_stack.push(UndoType::Loaded);
                self.prev_l_layers.push(self.loaded_layers.clone());
                
                for (layer_name, is_checked) in &self.checkbox_for_layer{
                    if !is_checked {
                        //out_map.insert(layer_name.clone(), self.loaded_layers.get(layer_name).unwrap().clone());
                        continue;
                    }
                    self.loaded_layers.remove(layer_name);
                }
                    self.current_svg = svgwrite::create_svg(&self.loaded_layers, &self.min_x, &self.max_y, &self.width, &self.height);
                    self.svg_image = egui_extras::RetainedImage::from_svg_bytes_with_size(
                    "test", //path of svg file to display
                    self.current_svg.to_string().as_bytes(), 
                    FitTo::Size(3840, 2160), //display resolution (need to check performance effect)
                    )
                    .unwrap();
            }
            
        });
        egui::TopBottomPanel::top("top_panel").frame(_my_frame).show(ctx, |ui|{
            ui.horizontal(|ui|{
                ui.heading("File Selector");
                //ui.set_min_size(ui.available_size());
                if ui.button("Open file…").clicked() {
                    if let Some(path) = rfd::FileDialog::new().add_filter("dxf", &["dxf"]).pick_file() {
                        self.picked_path = Some(path.display().to_string());
                        
                        //get extension to see if we want to update display
                        let extension = path.extension().unwrap();
                        if extension == "dxf"{
                            //self.selected = true;
                            //self.previous_dxfs = Vec::<Drawing>::new();
                            //self.next_dxfs = Vec::<Drawing>::new();
                            //self.previous_svgs = Vec::<svg::Document>::new();
                            //self.next_svgs = Vec::<svg::Document>::new();
                            //if we want to be able to undo to old opened files we need to fix something right here
                            self.prev_l_layers = Vec::<HashMap<String, Vec<PolyLine>>>::default();
                            self.next_l_layers = Vec::<HashMap<String, Vec<PolyLine>>>::default();
                            self.prev_c_layers = Vec::<HashMap<String, Vec<PolyLine>>>::default();
                            self.next_c_layers = Vec::<HashMap<String, Vec<PolyLine>>>::default();
                            self.loaded_dxf = dxf::Drawing::load_file(self.picked_path.clone().unwrap()).expect("Not a valid file");
                            let mut layer_polylines = HashMap::<String, Vec<PolyLine>>::default();
                            let layers = dxfextract::extract_layers(&self.loaded_dxf);
                            let mut checkbox_map = HashMap::<String, bool>::default();
                            let mut old_name_map = HashMap::<String, String>::default();

                            for (name, layer) in layers.iter() {
                                layer_polylines.insert(name.clone(), layer.into_polylines());
                                //layer_color.insert(name.clone(), colors.pop().unwrap().to_owned());
                            }

                            self.loaded_layers = layer_polylines.clone();
                            self.current_layers = layer_polylines.clone();

                            for layer_name in self.loaded_layers.keys() {
                                //println!("{}", layer_name);
                                checkbox_map.insert(layer_name.clone(), true);
                                old_name_map.insert(layer_name.clone(), layer_name.clone());
                            }

                            self.checkbox_for_layer = checkbox_map;
                            self.old_to_new_name = old_name_map;

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

                        ui.separator();


                        }
                    }
                }                
                
            });
            
            if let Some(picked_path) = &self.picked_path {
                ui.horizontal(|ui| {
                    ui.label("Chosen file:");
                    ui.monospace(picked_path);
                });
            }
            
            
            //SAVE BUTTONS
            ui.horizontal(|ui| {
            if ui.button("Save").clicked() {
                if !&self.picked_path.clone().unwrap().eq("") {
                    let res = rfd::FileDialog::new().set_file_name("export").set_directory(&self.picked_path.clone().unwrap()).add_filter("dxf", &["dxf"]).add_filter("svg", &["svg"]).save_file();
                    
                    if let Some(extension) = res{
                        let filetype = extension.extension().unwrap(); //get extension
                        let filepath = extension.as_path().as_os_str().to_os_string().into_string().unwrap(); //convert from &OsStr to String

                        //save dxf
                        if filetype == "dxf"{
                            let mut out_layers = HashMap::<String, Vec<PolyLine>>::default();
                            for (layer_name, polylines) in &self.current_layers{
                                out_layers.insert(self.old_to_new_name.get(layer_name).unwrap().clone(), polylines.clone());
                            }
                            match dxfwrite::savedxf(out_layers, &filepath){
                                Ok(_) => info!("DXF saved!"),
                                Err(err) => panic!("Error while saving DXF: {}", err),
                            };
                        }
                        //save svg
                        else if filetype == "svg"{
                            svgwrite::save_svg(&filepath, &self.current_svg);
                        }
                        //pop-up message error
                        else{
                            let _msg = rfd::MessageDialog::new().set_title("Error!").set_description("Something went wrong while saving. Did you chose the correct extension?").set_buttons(rfd::MessageButtons::Ok).show();
                        
                        }
                    }
                    
                    
                    
                    



                }
                
            }
            
        });

        });
        //ui the last panel added. this one should only contain our svg if we decide to use multiple panels down the line
        egui::CentralPanel::default().frame(_my_frame).show(ctx, |ui| {
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

