#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod dxfwrite;
mod algorithms;
mod svgwrite;
mod dxfextract;
use dxfextract::PolyLine;
use eframe::{egui};
use egui_extras::image::FitTo;
use egui::{Color32, ScrollArea, Vec2};
use std::sync::{RwLock, mpsc::{Receiver, Sender}};
use svg::Document;
use std::{collections::{BTreeMap}};
use log::{error, info, warn};
use egui::{Slider};
use std::time::Duration;
use tokio::runtime::Runtime;
use egui::FontFamily::Proportional;
use egui::FontId;
use egui::TextStyle::*;
//use egui::widgets::Button;
use egui::menu;

enum UndoType {
    //hard type - change in layers loaded
    Loaded,
    //soft type - change in layers shown only
    Current,
}
//constants
const DEFAULT_MERGE_NAME: &str = "merge_layer";
const MAX_ZOOM: i32 = 10;

#[derive(Clone)]
struct RawOpen {
    polylines: BTreeMap<String, Vec<PolyLine>>,
    svg: Document,
    min_x: f64,
    max_y: f64,
    width: f64,
    height: f64,
}
impl RawOpen {
    pub fn new(polylines: BTreeMap<String, Vec<PolyLine>>, svg: Document, min_x: f64, max_y: f64, width: f64, height: f64) -> Self {
        Self {polylines, svg, min_x, max_y, width, height}
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

    //async handling
    let rt = Runtime::new().expect("Unable to create Runtime");

    // Enter the runtime so that `tokio::spawn` is available immediately.
    let _enter = rt.enter();

    // Execute the runtime in its own thread.
    // The future doesn't have to do anything. In this example, it just sleeps forever.
    std::thread::spawn(move || {
        rt.block_on(async {
            loop {
                tokio::time::sleep(Duration::from_secs(3600)).await;
            }
        })
    });

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


pub struct SvgApp {

    //thread communication
    connect_sender: Sender<BTreeMap<String, Vec<PolyLine>>>,
    connect_receiver: Receiver<BTreeMap<String, Vec<PolyLine>>>,
    open_sender: Sender<RawOpen>,
    open_receiver: Receiver<RawOpen>,

    //opened dxf-file
    
    picked_path: Option<String>,
    //loaded_dxf: Drawing,

    //info about the loaded file
    min_x: f64,
    max_y: f64,
    width: f64,
    height: f64,

    //document that gets saved using rust svg crate
    current_svg: RwLock<svg::Document>,
    svg_image: RwLock<egui_extras::RetainedImage>,

    //handles changes in the file where checkboxes get updated
    loaded_layers: BTreeMap<String, Vec<PolyLine>>,
    //handles changes in the file where checkboxes do not get updated
    current_layers: RwLock<BTreeMap<String, Vec<PolyLine>>>,
    //Handles the undo system
    undo_stack: Vec<UndoType>,
    redo_stack: Vec<UndoType>,
    prev_l_layers: Vec<BTreeMap<String, Vec<PolyLine>>>,
    next_l_layers: Vec<BTreeMap<String, Vec<PolyLine>>>,
    
    prev_c_layers: Vec<BTreeMap<String, Vec<PolyLine>>>,
    next_c_layers: RwLock<Vec<BTreeMap<String, Vec<PolyLine>>>>,
    checkbox_for_layer: BTreeMap<String, bool>,
    //index for renaming
    old_to_new_name: BTreeMap::<String, String>,
    toggled: bool,
    last_toggled: bool,
    //slider info
    iterations_slider_value: i32,
    max_angle_slider_value: i32,
    max_distance_slider_value: i32,
    //changable name used when merging
    merge_name: String,
    //keys that are being pressed at any given time
    //pressed_keys: HashSet<egui::Key>,
    //handles zooming of the image
    current_zoom: f32,
    is_loading: RwLock<bool>,
    //instance: SvgApp,

    
}


impl Default for SvgApp {
    fn default() -> Self {
        let (connect_sender, connect_receiver) = std::sync::mpsc::channel();
        let (open_sender, open_receiver) = std::sync::mpsc::channel();
        Self {
            connect_sender,
            connect_receiver,
            open_sender,
            open_receiver,
            svg_image: RwLock::new(egui_extras::RetainedImage::from_svg_bytes_with_size(
                "../tmp_file.svg", //path of svg file to display
                include_bytes!("../tmp_file.svg"), 
                FitTo::Size(3840, 2160), //display resolution (need to check performance effect)
            )
            .unwrap()),
            picked_path: Some("".to_string()),
            min_x: 0.0,
            max_y: 0.0,
            width: 0.0,
            height: 0.0,
            current_svg: RwLock::new(Document::new()),
            loaded_layers: BTreeMap::<String, Vec<PolyLine>>::default(),
            undo_stack: Vec::<UndoType>::default(),
            redo_stack: Vec::<UndoType>::default(),
            prev_l_layers: Vec::<BTreeMap<String, Vec<PolyLine>>>::default(),
            next_l_layers: Vec::<BTreeMap<String, Vec<PolyLine>>>::default(),
            current_layers: RwLock::new(BTreeMap::<String, Vec<PolyLine>>::default()),
            prev_c_layers: Vec::<BTreeMap<String, Vec<PolyLine>>>::default(),
            next_c_layers: RwLock::new(Vec::<BTreeMap::<String, Vec<PolyLine>>>::default()),
            checkbox_for_layer: BTreeMap::<String, bool>::default(),
            old_to_new_name: BTreeMap::<String, String>::default(),
            toggled: true,
            last_toggled: true,
            iterations_slider_value: 1,
            max_angle_slider_value: 360,
            max_distance_slider_value: 1000,
            merge_name: DEFAULT_MERGE_NAME.to_string(),
            //pressed_keys: HashSet::<egui::Key>::default(),
            current_zoom: 1.0,
            is_loading: RwLock::new(false),
            //instance: SvgApp::default(),
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
        let mut style = (*ctx.style()).clone();
        style.text_styles = [
            (Heading, FontId::new(30.0, Proportional)),
            (Name("Heading2".into()), FontId::new(25.0, Proportional)),
            (Name("Context".into()), FontId::new(16.0, Proportional)),
            (Body, FontId::new(15.0, Proportional)),
            (Monospace, FontId::new(14.0, Proportional)),
            (Button, FontId::new(15.0, Proportional)),
            (Small, FontId::new(10.0, Proportional)),
        ].into();
        ctx.set_style(style);
        //when calculations are done when using connect
        if let Ok(response) = self.connect_receiver.try_recv() {
            finished_connect(self, response);
        }
        //when calculations are done after opening file
        if let Ok(response) = self.open_receiver.try_recv() {
            finished_open(self, response);
        }
        //key and input handling
        ctx.input(|i| {
            
            //keybinds with alt as modifier
            if i.modifiers.alt {
                //checks if user is scrolling
                let scroll = i.scroll_delta.y;
                if scroll != 0. {
                    if scroll > 0. {
                        zoom_in(self);
                    }
                    else {
                        zoom_out(self);
                    }
                }
            }
            //keybinds with ctrl as modifier
            else if i.modifiers.ctrl {
                //opens open dialogue
                if i.modifiers.shift {
                    if i.keys_down.contains(&egui::Key::Z){
                        redo(self);
                    }
                }
                else if i.keys_down.contains(&egui::Key::O){
                    open_file(self, ctx.clone());
                }
                //opens save dialogue
                else if i.keys_down.contains(&egui::Key::S){
                    save_file(self, ctx.clone());
                }
                else if i.keys_down.contains(&egui::Key::Z){
                    undo(self);
                }
                else if i.keys_down.contains(&egui::Key::PlusEquals){
                    zoom_in(self);
                }
                else if i.keys_down.contains(&egui::Key::Minus){
                    zoom_out(self);
                }
            }
        });
        

        egui::SidePanel::right("right_panel").frame(_my_frame).show(ctx, |ui|{
            ui.heading("Tools");
            ui.separator();
            ui.set_min_size(ui.available_size());
            ui.add_space(ui.spacing().item_spacing.y); // Add line space here

                ui.vertical(|ui|{

                    let button8 = egui::Button::new("Connect lines");
                    let minsize: Vec2 = [70.0, 25.0].into ();    

                if ui.add(button8.min_size(minsize)).clicked()&& !*self.is_loading.read().unwrap(){
                    self.undo_stack.push(UndoType::Current);
                    self.prev_c_layers.push(self.current_layers.read().unwrap().clone());
                    let mut temp = BTreeMap::<String, Vec<PolyLine>>::default();
                    //let mut counter = 0;
                    for (name, checked) in &self.checkbox_for_layer {
                        if checked.clone(){
                            temp.insert(name.clone(), self.loaded_layers.get(name).unwrap().clone());
                            //counter += 1;
                        }
                    }
                    *self.is_loading.write().unwrap() = true;
                    
                    start_thread_connect(self.connect_sender.clone(), ctx.clone(), false, self.current_layers.read().unwrap().clone(), 
                    temp, Some((self.max_distance_slider_value as f64) / 1000. * f64::sqrt(self.width * self.width + self.height * self.height)), 
                    Some(self.max_angle_slider_value), Some(self.iterations_slider_value));
                }

                ui.add_space(ui.spacing().item_spacing.y); // Add line space here

                let button9 = egui::Button::new("Extend lines");
                let minsize: Vec2 = [70.0, 25.0].into ();
    
                if ui.add(button9.min_size(minsize)).clicked()&& !*self.is_loading.read().unwrap(){
                    self.undo_stack.push(UndoType::Current);
                    self.prev_c_layers.push(self.current_layers.read().unwrap().clone());
                    let mut temp = BTreeMap::<String, Vec<PolyLine>>::default();
                    //let mut counter = 0;
                    for (name, checked) in &self.checkbox_for_layer {
                        if checked.clone(){
                            temp.insert(name.clone(), self.loaded_layers.get(name).unwrap().clone());
                            //counter += 1;
                        }
                    }
                    *self.is_loading.write().unwrap() = true;
                    
                    start_thread_connect(self.connect_sender.clone(), ctx.clone(), true, self.current_layers.read().unwrap().clone(), 
                    temp, Some((self.max_distance_slider_value as f64) / 1000. * f64::sqrt(self.width * self.width + self.height * self.height)), 
                    Some(self.max_angle_slider_value), Some(self.iterations_slider_value));
                    
                }                    
            });
            ui.add_space(ui.spacing().item_spacing.y); // Add line space here
            ui.separator();
            ui.add_space(ui.spacing().item_spacing.y); // Add line space here

            // SLIDERS
            // wrap the slider in a vertical layout to move it to a new line
            ui.vertical(|ui| {
            //ui.add(egui::Label::new("Iterations"));
                ui.add(Slider::new(&mut self.iterations_slider_value, 1..=10).text("Iterations (amount)"));
                ui.add(Slider::new(&mut self.max_distance_slider_value, 1..=1000).text("Max distance (‰)"));
                ui.add(Slider::new(&mut self.max_angle_slider_value, 1..=180).text("Max angle (°)"));
            // do not update value with slider_value when slider is change
            });

            // assign the final slider value to value after the UI is drawn

            
            ui.separator();
            ui.add_space(ui.spacing().item_spacing.y); // Add line space here
            ui.checkbox(&mut self.toggled, "Toggle All On/Off");
            ui.add_space(ui.spacing().item_spacing.y); // Add line space here
            ui.separator();
            let mut checkboxes = BTreeMap::<String, bool>::default();
            let mut new_layer_names = BTreeMap::<String, String>::default();
            
            //List of layers in sidepanel
            egui::ScrollArea::vertical().max_height(500.0).show(ui, |ui|{
                for (layer_name, _polylines)in self.loaded_layers.clone() {
                    let mut checkval = self.checkbox_for_layer.get(&layer_name).unwrap().clone();
                    //let mut new_name = layer_name.clone();
                    //println!("{}", &layer_name);
                    let mut new_name = self.old_to_new_name.get(&layer_name).unwrap().clone();
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
                    let mut checkboxes = BTreeMap::<String, bool>::default();
                    for layer_name in self.loaded_layers.keys() {
                        checkboxes.insert(layer_name.clone(), self.toggled);
                    }
                    self.checkbox_for_layer = checkboxes;
                }
                self.last_toggled = self.toggled;
                ui.add_space(ui.spacing().item_spacing.y); // Add line space here
    
            });

            ui.add_space(ui.spacing().item_spacing.y); // Add line space here
            self.last_toggled = self.toggled;
            ui.horizontal(|ui|{

                let button6 = egui::Button::new("Merge layer(s)");
                let minsize: Vec2 = [70.0, 25.0].into ();

                if ui.add(button6.min_size(minsize)).clicked() {
                    //checks wheter the name is in use or not
                        let mut full_layer = Vec::<PolyLine>::default();
                        if self.merge_name == "".to_string() || self.loaded_layers.contains_key(&self.merge_name) && !self.checkbox_for_layer.get(&self.merge_name).unwrap(){
                            let _msg = rfd::MessageDialog::new().set_title("Error!").set_description("The new layer needs different name!").set_buttons(rfd::MessageButtons::Ok).show();
                        }
                        else{
                            self.undo_stack.push(UndoType::Loaded);
                            self.prev_l_layers.push(self.loaded_layers.clone());
                            let mut counter = 0;
                            for (layer_name, is_checked) in &self.checkbox_for_layer{
                                if !is_checked {
                                    continue;
                                }
                                counter += 1;
                                full_layer.append(&mut self.loaded_layers.get(layer_name).unwrap().clone());
                                self.loaded_layers.remove(layer_name);
                            }
                            self.loaded_layers.insert(self.merge_name.clone(), full_layer);
                            self.checkbox_for_layer.insert(self.merge_name.clone(), true);
                            let mut temp = BTreeMap::<String, Vec<PolyLine>>::default();
                            for (name, val) in &self.loaded_layers {
                                if self.checkbox_for_layer.get(name).unwrap().clone() {
                                    temp.insert(name.clone(), val.clone());
                                }
                            }
                            *self.current_layers.write().unwrap() = temp;
                            self.old_to_new_name.insert(self.merge_name.clone(), self.merge_name.clone());
                            self.merge_name = DEFAULT_MERGE_NAME.to_string();
                            *self.current_svg.write().unwrap() = svgwrite::create_svg(&self.current_layers.read().unwrap().clone(), &self.min_x, &self.max_y, &self.width, &self.height);
                            *self.svg_image.write().unwrap() = render_svg(&self.current_svg.read().unwrap());

                            info!("Merged {} layers", counter);
                        }         
                }
                ui.add(egui::TextEdit::singleline(&mut self.merge_name));
            });
            ui.add_space(ui.spacing().item_spacing.y); // Add line space here
            ui.add_space(ui.spacing().item_spacing.y); // Add line space here
            
            self.last_toggled = self.toggled;
            let button5 = egui::Button::new("Update visuals");
            let minsize: Vec2 = [70.0, 25.0].into ();
            
            if ui.add(button5.min_size(minsize)).clicked() {
                let mut out_layers_name = BTreeMap::<String, Vec<PolyLine>>::default();
                let mut old_name_map = BTreeMap::<String, String>::default();
                for (name, val) in self.loaded_layers.clone() {
                    let mut new_name = self.old_to_new_name.get(&name).unwrap().clone();
                    while out_layers_name.contains_key(&new_name){
                        new_name.push('_');
                    }
                    if new_name != name {
                        self.checkbox_for_layer.insert(new_name.clone(), self.checkbox_for_layer.get(&name).unwrap().clone());
                        self.checkbox_for_layer.remove(&name);
                        out_layers_name.insert(new_name.clone(), val.clone());
                        old_name_map.insert(new_name.clone(), new_name.clone());
                    }
                    else{
                        out_layers_name.insert(name.clone(), val.clone());
                        old_name_map.insert(name.clone(), name.clone());
                    }
                }
                self.loaded_layers = out_layers_name;
                *self.current_layers.write().unwrap() = self.loaded_layers.clone();
                self.old_to_new_name = old_name_map;

                //rebuild part
                let mut temp = BTreeMap::<String, Vec<PolyLine>>::default();
                for (name, checked) in &self.checkbox_for_layer {
                    if checked.clone(){
                        temp.insert(name.clone(), self.loaded_layers.get(name).unwrap().clone());
                    }
                }
                self.undo_stack.push(UndoType::Current);
                self.prev_c_layers.push(self.current_layers.read().unwrap().clone());
                *self.current_layers.write().unwrap() = temp;
                *self.next_c_layers.write().unwrap() = Vec::<BTreeMap<String, Vec<PolyLine>>>::default();
                
                //TODO fix a better way to store previous files, so we can remove the first of them after a certain treshold
                //println!("Length of DXF-vector: {}", self.previous_dxfs.len());
                let mut out_layers = BTreeMap::<String, Vec<PolyLine>>::default();
                    for (layer_name, polylines) in self.current_layers.read().unwrap().clone(){
                        out_layers.insert(layer_name.clone(), polylines.clone());
                    }
                *self.current_svg.write().unwrap() = svgwrite::create_svg(&out_layers, &self.min_x, &self.max_y, &self.width, &self.height);
                *self.svg_image.write().unwrap() = render_svg(&self.current_svg.read().unwrap());

                info!("Rebuilt image");
            }
            ui.add_space(ui.spacing().item_spacing.y); // Add line space here
            ui.add_space(ui.spacing().item_spacing.y); // Add line space here

            ui.separator();
            let button7 = egui::Button::new("Delete layer(s)");
            let minsize: Vec2 = [70.0, 25.0].into ();

            if ui.add(button7.min_size(minsize)).clicked() {
                delete_layer(self);
            }
            
        });
        
        /*egui::TopBottomPanel::top("top_panel").frame(_my_frame).show(ctx, |ui|{
            

        });*/
        //ui the last panel added. this one should only contain our svg if we decide to use multiple panels down the line
        egui::CentralPanel::default().frame(_my_frame).show(ctx, |ui| {
            menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    ui.set_max_width(240.0);
                    //ui.set_style(egui::Style::default());
                    if ui.add(egui::Button::new("Open File...").shortcut_text("Ctrl + O")).clicked() {
                        open_file(self, ctx.clone());
                    }
                    
                    ui.separator();
                    if ui.add(egui::Button::new("Save File As...").shortcut_text("Ctrl + S")).clicked() {
                        save_file(self, ctx.clone());
                    }
                    ui.separator();
                    if ui.add(egui::Button::new("Undo").shortcut_text("Ctrl + Z")).clicked() {
                        undo(self);
                    }
                    ui.separator();
                    if ui.add(egui::Button::new("Redo").shortcut_text("Ctrl + Shift + Z")).clicked() {
                        redo(self);
                    }

                });
                
                ui.separator();
                ui.menu_button("Zoom", |ui| {
                    ui.set_max_width(240.0);
                    if ui.add(egui::Button::new("Zoom in").shortcut_text("Ctrl + +")).clicked() {
                        zoom_in(self);
                    }
                    ui.separator();
                    if ui.add(egui::Button::new("Zoom out").shortcut_text("Ctrl + -")).clicked() {
                        zoom_out(self);
                    }

                });
                ui.separator();
                ui.menu_button("Tools", |ui| {
                    ui.set_max_width(240.0);
                    if ui.button("Extend").clicked(){

                    }
                    ui.separator();
                    if ui.button("Connect").clicked(){
                        
                    }

                });
                ui.separator();
                ui.menu_button("Help", |ui| {
                    ui.set_max_width(240.0);
                });

            });
            ui.separator();
            if !self.is_loading.read().unwrap().clone() {
                ScrollArea::both().show(ui, |ui|{
                    self.svg_image.read().unwrap().show_scaled(ui, 0.4 * self.current_zoom); //0.4 original size because of the Resolution (High resolution ==> sharpness)
    
                });
            }
            else {
                ui.horizontal(|ui|{
                    ui.label("Loading");
                    ui.separator(); 
                    ui.spinner();
                });
                
            }
        });
        
    }
    
} 
fn start_thread_connect(tx: Sender<BTreeMap<String, Vec<PolyLine>>>, ctx: egui::Context, extend: bool, all_layers: BTreeMap<String, Vec<PolyLine>>, 
    affected_layers: BTreeMap<String, Vec<PolyLine>>, max_distance_in: Option<f64>, max_angle_in: Option<i32>, o_iterations: Option<i32>) {
    tokio::spawn(async move {
        info!("Started calculations!");
        // Send a request with an increment value.
        let calculated = algorithms::try_to_close_polylines(extend, &all_layers, &affected_layers, &max_distance_in, &max_angle_in, &o_iterations);
        
        // After parsing the response, notify the GUI thread of the increment value.
        let _ = tx.send(calculated);
        ctx.request_repaint();
    });
}
fn finished_connect(app: &mut SvgApp, response: BTreeMap<String, Vec<PolyLine>>) {
    info!("Connect done!");
    *app.current_layers.write().unwrap() = response;
    let mut out_layers = BTreeMap::<String, Vec<PolyLine>>::default();
    for (layer_name, polylines) in app.current_layers.read().unwrap().clone(){
        out_layers.insert(layer_name.clone(), polylines.clone());
    }
    *app.current_svg.write().unwrap() = svgwrite::create_svg(&out_layers, &app.min_x, &app.max_y, &app.width, &app.height);
    *app.next_c_layers.write().unwrap() = Vec::<BTreeMap<String, Vec<PolyLine>>>::default();
    *app.svg_image.write().unwrap() = render_svg(&app.current_svg.read().unwrap());
    *app.is_loading.write().unwrap() = false;
}
fn open_file(app: &mut SvgApp, ctx: egui::Context) {
    if let Some(path) = rfd::FileDialog::new().add_filter("dxf", &["dxf"]).pick_file() {
        app.picked_path = Some(path.display().to_string());
        //get extension to see if we want to update display
        let extension = path.extension().unwrap();
        if extension == "dxf" && !app.is_loading.read().unwrap().clone(){
            //if we want to be able to undo to old opened files we need to fix something right here
            *app.is_loading.write().unwrap() = true;
            app.prev_l_layers = Vec::<BTreeMap<String, Vec<PolyLine>>>::default();
            app.next_l_layers = Vec::<BTreeMap<String, Vec<PolyLine>>>::default();
            app.prev_c_layers = Vec::<BTreeMap<String, Vec<PolyLine>>>::default();
            *app.next_c_layers.write().unwrap() = Vec::<BTreeMap<String, Vec<PolyLine>>>::default();
            open_file_async(app.open_sender.clone(), ctx, path.display().to_string());
        }
    }
}
fn open_file_async(tx: Sender<RawOpen>, ctx: egui::Context, dxf_path: String) {
    tokio::spawn(async move {
        info!("Started opening file!");
        let dxf = dxf::Drawing::load_file(dxf_path).expect("Not a valid file");
        let mut layer_polylines = BTreeMap::<String, Vec<PolyLine>>::default(); 
                  
        let layers = dxfextract::extract_layers(&dxf);
        for (name, layer) in layers.iter() {
            layer_polylines.insert(name.clone(), layer.into_polylines());
        }

        let min_x;
        let max_y;
        let width;
        let height;
        if let Some(result) = algorithms::calculate_min_max(&layer_polylines) {
            min_x = result.0;
            max_y = result.2;
            width = result.3;
            height = result.4;
        }
        else {
            panic!("Calculate_min_max not working!");
        }
        let svg = svgwrite::create_svg(&layer_polylines, &min_x, &max_y, &width, &height);
        let raw = RawOpen::new(layer_polylines, svg, min_x, max_y, width, height);
        let _ = tx.send(raw);
        ctx.request_repaint();
    });
    
}
fn finished_open(app: &mut SvgApp, response: RawOpen) {
    populate_maps(app, response.polylines.clone());

    app.loaded_layers = response.polylines.clone();
    *app.current_layers.write().unwrap() = response.polylines;

    *app.current_svg.write().unwrap() = response.svg;
    *app.svg_image.write().unwrap() = render_svg(&app.current_svg.read().unwrap());
    app.min_x = response.min_x;
    app.max_y = response.max_y;
    app.width = response.width;
    app.height = response.height;
    *app.is_loading.write().unwrap() = false;
    info!("Opened new file!");
}
fn save_file(app: &mut SvgApp, ctx: egui::Context) {
    if !&app.picked_path.clone().unwrap().eq("") {
        let res = rfd::FileDialog::new().set_file_name("export").set_directory(&app.picked_path.clone().unwrap()).add_filter("dxf", &["dxf"]).add_filter("svg", &["svg"]).save_file();
        
        if let Some(extension) = res{
            let filetype = extension.extension().unwrap(); //get extension
            let filepath = extension.as_path().as_os_str().to_os_string().into_string().unwrap(); //convert from &OsStr to String

            //save dxf
            if filetype == "dxf"{
                let mut out_layers = BTreeMap::<String, Vec<PolyLine>>::default();
                for (layer_name, polylines) in app.current_layers.read().unwrap().clone(){
                    out_layers.insert(layer_name.clone(), polylines.clone());
                }
                match dxfwrite::savedxf(out_layers, &filepath){
                    Ok(_) => info!("DXF saved!"),
                    Err(err) => panic!("Error while saving DXF: {}", err),
                };
            }
            //save svg
            else if filetype == "svg"{
                svgwrite::save_svg(&filepath, &app.current_svg.read().unwrap());
            }
            //pop-up message error
            else{
                let _msg = rfd::MessageDialog::new().set_title("Error!").set_description("Something went wrong while saving. Did you chose the correct extension?").set_buttons(rfd::MessageButtons::Ok).show();
            
            }
        }
        
        
    }
}
fn delete_layer(app: &mut SvgApp) {
    let _msg = rfd::MessageDialog::new().set_title("ALERT!").set_description("Are you sure you want to delete this layer(s)").set_buttons(rfd::MessageButtons::OkCancel).show();
    if !_msg{
        //do not do anything, cancel delete
    }
    else{
        app.undo_stack.push(UndoType::Loaded);
        app.prev_l_layers.push(app.loaded_layers.clone());
        let mut counter = 0;
        for (layer_name, is_checked) in &app.checkbox_for_layer{
            if !is_checked {
                continue;
            }
            counter += 1;
            app.loaded_layers.remove(layer_name);
        }
        *app.current_svg.write().unwrap() = svgwrite::create_svg(&app.loaded_layers, &app.min_x, &app.max_y, &app.width, &app.height);
        *app.svg_image.write().unwrap() = render_svg(&app.current_svg.read().unwrap());

        info!("Deleted {} layers", counter);
    }
}
fn undo(app: &mut SvgApp) {
    if let Some(undo_type) = app.undo_stack.pop() {
        match undo_type {
            UndoType::Loaded => {
                if let Some(prev) = app.prev_l_layers.pop() {
                    app.redo_stack.push(UndoType::Loaded);
                    app.next_l_layers.push(app.loaded_layers.clone());
                    app.loaded_layers = prev;
                    populate_maps(app, app.loaded_layers.clone());
                    *app.current_svg.write().unwrap() = svgwrite::create_svg(&app.loaded_layers, &app.min_x, &app.max_y, &app.width, &app.height);
                    *app.svg_image.write().unwrap() = render_svg(&app.current_svg.read().unwrap());
                }
            },
            UndoType::Current => {
                if let Some(prev) = app.prev_c_layers.pop() {
                    app.redo_stack.push(UndoType::Current);
                    app.next_c_layers.write().unwrap().push(app.current_layers.read().unwrap().clone());
                    *app.current_layers.write().unwrap() = prev;
                    let mut out_layers = BTreeMap::<String, Vec<PolyLine>>::default();
                    for (layer_name, polylines) in app.current_layers.read().unwrap().clone(){
                        out_layers.insert(layer_name.clone(), polylines.clone());
                    }
                    *app.current_svg.write().unwrap() = svgwrite::create_svg(&out_layers, &app.min_x, &app.max_y, &app.width, &app.height);
                    *app.svg_image.write().unwrap() = render_svg(&app.current_svg.read().unwrap());

                    let mut temp = BTreeMap::<String, bool>::default();
                    for (name, _polylines) in &app.loaded_layers {
                        if app.current_layers.read().unwrap().contains_key(name){
                            temp.insert(name.clone(), true);
                            continue;
                        }
                        temp.insert(name.clone(), false);
                    }
                    app.checkbox_for_layer = temp;
                }
            },
        }
        info!("Undid 1 step");
    } 
}
fn redo(app: &mut SvgApp) {
    if let Some(undo_type) = app.redo_stack.pop() {
        match undo_type {
            UndoType::Loaded => {
                if let Some(next) = app.next_l_layers.pop(){
                    app.undo_stack.push(UndoType::Loaded);
                    app.prev_l_layers.push(app.loaded_layers.clone());
                    app.loaded_layers = next;
                    *app.current_svg.write().unwrap() = svgwrite::create_svg(&app.loaded_layers, &app.min_x, &app.max_y, &app.width, &app.height);
                    *app.svg_image.write().unwrap() = render_svg(&app.current_svg.read().unwrap());
                    populate_maps(app, app.loaded_layers.clone());
                }
            },
            UndoType::Current => {
                if let Some(next) = app.next_c_layers.write().unwrap().pop(){
                    app.undo_stack.push(UndoType::Current);
                    app.prev_c_layers.push(app.current_layers.read().unwrap().clone());
                    *app.current_layers.write().unwrap() = next;
                    let mut out_layers = BTreeMap::<String, Vec<PolyLine>>::default();
                    for (layer_name, polylines) in app.current_layers.read().unwrap().clone(){
                        out_layers.insert(layer_name.clone(), polylines.clone());
                    }
                    *app.current_svg.write().unwrap() = svgwrite::create_svg(&out_layers, &app.min_x, &app.max_y, &app.width, &app.height);
                    *app.svg_image.write().unwrap() = render_svg(&app.current_svg.read().unwrap());
                    let mut temp = BTreeMap::<String, bool>::default();
                    for (name, _polylines) in &app.loaded_layers {
                        if app.current_layers.read().unwrap().contains_key(name){
                            temp.insert(name.clone(), true);
                            continue;
                        }
                        temp.insert(name.clone(), false);
                    }
                    app.checkbox_for_layer = temp;
                }
            },
        }
        info!("Redid 1 step");
    }
}
fn zoom_in(app: &mut SvgApp) {
    if app.current_zoom < MAX_ZOOM as f32 {
        app.current_zoom += 0.1;
    }
}
fn zoom_out(app: &mut SvgApp) {
    if app.current_zoom > 0.2 {
        app.current_zoom -= 0.1;
    }
}

fn render_svg(svg: &Document) -> egui_extras::RetainedImage {
    let image = egui_extras::RetainedImage::from_svg_bytes_with_size(
        "rendered_image", //path of svg file to display
        svg.to_string().as_bytes(), 
        FitTo::Size(3840, 2160), //display resolution (need to check performance effect)
    )
    .unwrap();
    info!("Rendered new svg");
    image
}
fn populate_maps(app: &mut SvgApp, polylines: BTreeMap<String, Vec<PolyLine>>) {
    let mut checkbox_map = BTreeMap::<String, bool>::default();
    let mut old_name_map = BTreeMap::<String, String>::default();
    for layer_name in polylines.keys() {
        checkbox_map.insert(layer_name.clone(), true);
        old_name_map.insert(layer_name.clone(), layer_name.clone());
    }
        
    app.checkbox_for_layer = checkbox_map;
    app.old_to_new_name = old_name_map;
}


