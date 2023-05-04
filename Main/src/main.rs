#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod dxfwrite;
mod algorithms;
mod svgwrite;
mod dxfextract;
use dxf::Color;
use dxfextract::PolyLine;
use eframe::{egui};
use egui_extras::image::FitTo;
use egui::{Color32, ScrollArea, Vec2, layers, Image};
use std::{sync::{RwLock, mpsc::{Receiver, Sender}}, path::PathBuf, default};
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
    min_x: f64,
    max_y: f64,
    width: f64,
    height: f64,
}
impl RawOpen {
    pub fn new(polylines: BTreeMap<String, Vec<PolyLine>>, min_x: f64, max_y: f64, width: f64, height: f64) -> Self {
        Self {polylines, min_x, max_y, width, height}
    }
}
//#[derive(Clone)]
struct RawSvg {
    svg: Document,
    image: egui_extras::RetainedImage,
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
    save_sender: Sender<bool>,
    save_receiver: Receiver<bool>,
    render_sender: Sender<RawSvg>,
    render_receiver: Receiver<RawSvg>,

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
    next_c_layers: Vec<BTreeMap<String, Vec<PolyLine>>>,
    checkbox_for_layer: BTreeMap<String, bool>,
    color_for_layer: BTreeMap::<String, [f32; 3]>,
    last_checkbox_for_layer: BTreeMap<String, bool>,
    last_color_for_layer: BTreeMap::<String, [f32; 3]>,
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
        let (save_sender, save_receiver) = std::sync::mpsc::channel();
        let (render_sender, render_receiver) = std::sync::mpsc::channel();
        Self {
            connect_sender,
            connect_receiver,
            open_sender,
            open_receiver,
            save_sender,
            save_receiver,
            render_sender,
            render_receiver,
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
            next_c_layers: Vec::<BTreeMap::<String, Vec<PolyLine>>>::default(),
            checkbox_for_layer: BTreeMap::<String, bool>::default(),
            color_for_layer: BTreeMap::<String, [f32; 3]>::default(),
            last_checkbox_for_layer: BTreeMap::<String, bool>::default(),
            last_color_for_layer: BTreeMap::<String, [f32; 3]>::default(),
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
            finished_connect(self, ctx.clone(), response);
        }
        //when calculations are done after opening file
        if let Ok(response) = self.open_receiver.try_recv() {
            finished_open(self, ctx.clone(), response);
        }
        if let Ok(response) = self.save_receiver.try_recv() {
            finished_save(self, response);
        }
        if let Ok(response) = self.render_receiver.try_recv(){
            finished_render(self, response);
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
                        redo(self, ctx.clone());
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
                    undo(self, ctx.clone());
                }
                else if i.keys_down.contains(&egui::Key::PlusEquals){
                    zoom_in(self);
                }
                else if i.keys_down.contains(&egui::Key::Minus){
                    zoom_out(self);
                }
            }
        });
        

        egui::SidePanel::right("right_panel").resizable(false).frame(_my_frame).show(ctx, |ui|{
            ui.heading("Tools");
            ui.separator();
            ui.set_min_size(ui.available_size());
            ui.add_space(ui.spacing().item_spacing.y); // Add line space here
            ui.visuals_mut().override_text_color = Some(Color32::WHITE);

                ui.vertical(|ui|{

                    let button8 = egui::Button::new("Connect lines");
                    let minsize: Vec2 = [70.0, 25.0].into ();    

                if ui.add(button8.min_size(minsize)).clicked()&& !*self.is_loading.read().unwrap(){
                    prepare_connect(self, ctx.clone(), false);
                }

                ui.add_space(ui.spacing().item_spacing.y); // Add line space here

                let button9 = egui::Button::new("Extend lines");
                let minsize: Vec2 = [70.0, 25.0].into ();
    
                if ui.add(button9.min_size(minsize)).clicked()&& !*self.is_loading.read().unwrap(){
                    prepare_connect(self, ctx.clone(), true);
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
            
            //let mut colors = vec!["purple(16)","navy(16)","seagreen","darkslategrey","black","darkorchid","indianred","darkolivegreen","forestgreen", "indigo", "pink", "olive", "lightsalmon", "cornflowerblue", "deepskyblue", "brown", "darkred", "chocolate", "blueviolet", "purple", "orange", "green", "blue", "red"];
            //List of layers in sidepanel
            //also handling update of checkboxes and renaming here
            egui::ScrollArea::vertical().max_height(500.0).show(ui, |ui|{
                let mut checkboxes = BTreeMap::<String, bool>::default();
                let mut colors =  BTreeMap::<String, [f32; 3]>::default();
                //let mut new_layer_names = BTreeMap::<String, String>::default();
                self.last_checkbox_for_layer = self.checkbox_for_layer.clone();
                self.last_color_for_layer = self.color_for_layer.clone();
                let mut temp = BTreeMap::<String, String>::default();
                for (layer, polylines) in self.loaded_layers.clone() {
                    let mut checkval = self.checkbox_for_layer.get(&layer).unwrap().clone();
                    let mut color = self.color_for_layer.get(&layer).unwrap().clone();
                    //let mut new_name = layer.clone();
                    let mut new_name = self.old_to_new_name.get(&layer).unwrap().clone();
                    let mut status = false;
                    ui.horizontal(|ui|{
                        ui.checkbox(&mut checkval, "");
                        status = ui.text_edit_singleline(&mut new_name).lost_focus();
                        ui.color_edit_button_rgb(&mut color);
                    });
                    
                    if status && new_name != layer {
                        while self.loaded_layers.contains_key(&new_name) {
                            new_name.push('_');
                        }
                        self.loaded_layers.remove(&layer);
                        self.loaded_layers.insert(new_name.clone(), polylines);
                        self.checkbox_for_layer.insert(new_name.clone(), self.checkbox_for_layer.get(&layer).unwrap().clone());
                        self.checkbox_for_layer.remove(&layer);
                        self.color_for_layer.insert(new_name.clone(), self.color_for_layer.get(&layer).unwrap().clone());
                        self.color_for_layer.remove(&layer);
                        self.old_to_new_name.insert(new_name.clone(), new_name.clone());
                        self.old_to_new_name.remove(&layer);
                        temp.insert(new_name.clone(), new_name.clone());
                        colors.insert(new_name.clone(), color);
                        checkboxes.insert(new_name, checkval);
                    }
                    else{
                        temp.insert(layer.clone(), new_name.clone());
                        colors.insert(layer.clone(), color);
                        checkboxes.insert(layer, checkval);
                        
                    }
                    //checkboxes.insert(new_name.clone(), checkval);
                   
                    //
                }
                self.old_to_new_name = temp;
                
                self.checkbox_for_layer = checkboxes;
                self.color_for_layer = colors;
                
                

                //code for toggle on/off for all layers
                if self.toggled != self.last_toggled {
                    let mut checkboxes = BTreeMap::<String, bool>::default();
                    for layer_name in self.loaded_layers.keys() {
                        checkboxes.insert(layer_name.clone(), self.toggled);
                    }
                    self.checkbox_for_layer = checkboxes;
                }
                if self.checkbox_for_layer != self.last_checkbox_for_layer || self.color_for_layer != self.last_color_for_layer{
                    auto_rebuild(self, ctx.clone());
                }
                self.last_checkbox_for_layer = self.checkbox_for_layer.clone();
                self.last_toggled = self.toggled;
                ui.add_space(ui.spacing().item_spacing.y); // Add line space here
    
            });

            ui.add_space(ui.spacing().item_spacing.y); // Add line space here
            self.last_toggled = self.toggled;
            ui.horizontal(|ui|{

                let button6 = egui::Button::new("Merge layer(s)");
                let minsize: Vec2 = [70.0, 25.0].into ();

                if ui.add(button6.min_size(minsize)).clicked() {
                    merge_layers(self, ctx.clone());
                }
                ui.add(egui::TextEdit::singleline(&mut self.merge_name));
            });
            ui.add_space(ui.spacing().item_spacing.y); // Add line space here
            ui.add_space(ui.spacing().item_spacing.y); // Add line space here
            
            self.last_toggled = self.toggled;
            /*let button5 = egui::Button::new("Update visuals");
            let minsize: Vec2 = [70.0, 25.0].into ();*/
            
            /*if ui.add(button5.min_size(minsize)).clicked() {
                
            }*/
            ui.add_space(ui.spacing().item_spacing.y); // Add line space here
            ui.add_space(ui.spacing().item_spacing.y); // Add line space here

            ui.separator();

            //Creating a Ui scope to specifically assign delete button with black text and red fill.
            ui.scope(|ui| {
                ui.visuals_mut().override_text_color = Some(Color32::WHITE);

                let button7 = egui::Button::new("Delete layer(s)");
                let minsize: Vec2 = [70.0, 25.0].into ();

                if ui.add(button7.min_size(minsize).fill(Color32::from_rgb(245, 22, 22))).clicked() {
                    delete_layer(self, ctx.clone());
                }
            });
            
            
        });
        
        /*egui::TopBottomPanel::top("top_panel").frame(_my_frame).show(ctx, |ui|{
            

        });*/
        //ui the last panel added. this one should only contain our svg if we decide to use multiple panels down the line
        egui::CentralPanel::default().frame(_my_frame).show(ctx, |ui| {
            menu::bar(ui, |ui| {
                ui.visuals_mut().override_text_color = Some(Color32::WHITE);
                ui.menu_button("File", |ui| {
                    ui.visuals_mut().override_text_color = Some(Color32::WHITE);
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
                        undo(self, ctx.clone());
                    }
                    ui.separator();
                    if ui.add(egui::Button::new("Redo").shortcut_text("Ctrl + Shift + Z")).clicked() {
                        redo(self, ctx.clone());
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
//handles checkboxing and renaming
fn auto_rebuild(app: &mut SvgApp, ctx: egui::Context) {
    let mut out = BTreeMap::<String, Vec<PolyLine>>::default();
    let mut colors = Vec::<[f32; 3]>::default();
    for (name, val) in app.loaded_layers.clone() {
        if app.checkbox_for_layer.get(&name).unwrap().clone() {
            colors.push(app.color_for_layer.get(&name).unwrap().clone());
            out.insert(name, val);
        }
    }
    *app.current_layers.write().unwrap() = out.clone();
    render_svg(app, ctx, out, colors);
    //info!("Rebuilt image");
}

fn render_svg(app: &mut SvgApp, ctx: egui::Context, layers: BTreeMap<String, Vec<PolyLine>>, colors: Vec<[f32; 3]>) {
    *app.is_loading.write().unwrap() = true;
    render_async(app.render_sender.clone(), ctx, layers, app.min_x.clone(), app.max_y.clone(), app.width.clone(), app.height.clone(), colors);
}
fn render_async(tx: Sender<RawSvg>, ctx: egui::Context, layers: BTreeMap<String, Vec<PolyLine>>, min_x: f64, max_y: f64, width: f64, height: f64, colors: Vec<[f32; 3]>) {
    tokio::spawn(async move{
        let svg = svgwrite::create_svg(&layers, &min_x, &max_y, &width, &height, colors);
        let image = egui_extras::RetainedImage::from_svg_bytes_with_size(
            "rendered_image", //path of svg file to display
            svg.to_string().as_bytes(), 
            FitTo::Size(3840, 2160), //display resolution (need to check performance effect)
        )
        .unwrap();
        _ = tx.send(RawSvg { svg, image});
        ctx.request_repaint();
    });
}
fn finished_render(app: &mut SvgApp, response: RawSvg) {
    *app.current_svg.write().unwrap() = response.svg;
    *app.svg_image.write().unwrap() = response.image;
    *app.is_loading.write().unwrap() = false;
    info!("Rendered new svg");
}
fn prepare_connect(app: &mut SvgApp, ctx: egui::Context, extend: bool) {
    let mut out = BTreeMap::<String, Vec<PolyLine>>::default();
    for (name, val) in app.loaded_layers.clone() {
        if app.checkbox_for_layer.get(&name).unwrap().clone() {
            out.insert(name, val);
        }
    }
    app.undo_stack.push(UndoType::Loaded);
    app.prev_l_layers.push(app.loaded_layers.clone());
    *app.is_loading.write().unwrap() = true;
                    
    start_thread_connect(app.connect_sender.clone(), ctx, extend, app.loaded_layers.clone(), 
        app.current_layers.read().unwrap().clone(), Some((app.max_distance_slider_value as f64) / 1000. * f64::sqrt(app.width * app.width + app.height * app.height)), 
        Some(app.max_angle_slider_value), Some(app.iterations_slider_value));
                    
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
fn finished_connect(app: &mut SvgApp, ctx: egui::Context, response: BTreeMap<String, Vec<PolyLine>>) {
    app.loaded_layers = response;
    app.next_l_layers = Vec::<BTreeMap<String, Vec<PolyLine>>>::default();
    let mut out = BTreeMap::<String, Vec<PolyLine>>::default();
    for (name, val) in app.loaded_layers.clone() {
        if app.checkbox_for_layer.get(&name).unwrap().clone() {
            out.insert(name, val);
        }
    }
    *app.current_layers.write().unwrap() = out.clone();
    render_svg(app, ctx, out, app.color_for_layer.values().cloned().collect());
    *app.is_loading.write().unwrap() = false;
    info!("Connect done!");
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
            app.next_c_layers = Vec::<BTreeMap<String, Vec<PolyLine>>>::default();
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
        //let svg = svgwrite::create_svg(&layer_polylines, &min_x, &max_y, &width, &height);
        let raw = RawOpen::new(layer_polylines, min_x, max_y, width, height);
        let _ = tx.send(raw);
        ctx.request_repaint();
    });
    
}
fn finished_open(app: &mut SvgApp, ctx: egui::Context, response: RawOpen) {
    populate_maps(app, response.polylines.clone());

    app.loaded_layers = response.polylines.clone();
    app.min_x = response.min_x;
    app.max_y = response.max_y;
    app.width = response.width;
    app.height = response.height;
    *app.is_loading.write().unwrap() = false;
    render_svg(app, ctx, response.polylines.clone(), app.color_for_layer.values().cloned().collect());
    *app.current_layers.write().unwrap() = response.polylines;
    info!("Opened new file!");
}
fn save_file(app: &mut SvgApp, ctx: egui::Context) {
    if !&app.picked_path.clone().unwrap().eq("") {
        let res = rfd::FileDialog::new().set_file_name("export").set_directory(&app.picked_path.clone().unwrap()).add_filter("dxf", &["dxf"]).add_filter("svg", &["svg"]).save_file();
        if let Some(extension) = res{
            save_file_async(app.save_sender.clone(), ctx, extension, app.current_layers.read().unwrap().clone(), app.current_svg.read().unwrap().clone());
        }
    }
}
fn save_file_async(tx: Sender<bool>, ctx: egui::Context, extension: PathBuf, layers: BTreeMap<String, Vec<PolyLine>>, svg: Document) {
    tokio::spawn(async move {
        let filetype = extension.extension().unwrap(); //get extension
        let filepath = extension.as_path().as_os_str().to_os_string().into_string().unwrap(); //convert from &OsStr to String

        //save dxf
        if filetype == "dxf"{
            let mut out_layers = BTreeMap::<String, Vec<PolyLine>>::default();
            for (layer_name, polylines) in layers{
                out_layers.insert(layer_name.clone(), polylines.clone());
            }
            match dxfwrite::savedxf(out_layers, &filepath){
                Ok(_) => info!("DXF saved!"),
                Err(err) => panic!("Error while saving DXF: {}", err),
            };
            _ = tx.send(true);
        }
        //save svg
        else if filetype == "svg"{
            svgwrite::save_svg(&filepath, &svg);
            _ = tx.send(true);
        }
        //pop-up message error
        else{
            let _msg = rfd::MessageDialog::new().set_title("Error!").set_description("Something went wrong while saving. Did you chose the correct extension?").set_buttons(rfd::MessageButtons::Ok).show();
        }
        ctx.request_repaint();
    });
}
//code that will show a popup when save is finished
fn finished_save(app: &mut SvgApp, _: bool) {

}
fn delete_layer(app: &mut SvgApp, ctx: egui::Context) {
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
        populate_maps(app, app.loaded_layers.clone());
        render_svg(app, ctx, app.loaded_layers.clone(), app.color_for_layer.values().cloned().collect());

        info!("Deleted {} layers", counter);
    }
}
fn merge_layers(app: &mut SvgApp, ctx: egui::Context){
    //checks wheter the name is in use or not
    let mut full_layer = Vec::<PolyLine>::default();
    if app.merge_name == "".to_string() || app.loaded_layers.contains_key(&app.merge_name) && !app.checkbox_for_layer.get(&app.merge_name).unwrap(){
        let _msg = rfd::MessageDialog::new().set_title("Error!").set_description("The new layer needs different name!").set_buttons(rfd::MessageButtons::Ok).show();
    }
    else{
        app.undo_stack.push(UndoType::Loaded);
        app.prev_l_layers.push(app.loaded_layers.clone());
        let mut counter = 0;
        let mut temp = BTreeMap::<String, Vec<PolyLine>>::default();
        for (layer_name, is_checked) in app.checkbox_for_layer.clone().iter(){
            if !is_checked {
                continue;
            }
            temp.insert(layer_name.clone(), app.loaded_layers.get(layer_name).unwrap().clone());
            counter += 1;
            full_layer.append(&mut app.loaded_layers.get(layer_name).unwrap().clone());
            app.loaded_layers.remove(layer_name);
        }
        app.loaded_layers.insert(app.merge_name.clone(), full_layer);
        //app.checkbox_for_layer.insert(app.merge_name.clone(), true);
        
        *app.current_layers.write().unwrap() = temp;
        populate_maps(app, app.loaded_layers.clone());
        //app.old_to_new_name.insert(app.merge_name.clone(), app.merge_name.clone());
        app.merge_name = DEFAULT_MERGE_NAME.to_string();
        render_svg(app, ctx, app.loaded_layers.clone(), app.color_for_layer.values().cloned().collect());
        info!("Merged {} layers", counter);
    }         
}
fn undo(app: &mut SvgApp, ctx: egui::Context) {
    if let Some(undo_type) = app.undo_stack.pop() {
        match undo_type {
            UndoType::Loaded => {
                if let Some(prev) = app.prev_l_layers.pop() {
                    app.redo_stack.push(UndoType::Loaded);
                    app.next_l_layers.push(app.loaded_layers.clone());
                    app.loaded_layers = prev;
                    populate_maps(app, app.loaded_layers.clone());
                    render_svg(app, ctx, app.loaded_layers.clone(), app.color_for_layer.values().cloned().collect());
                }
            },
            UndoType::Current => {
                if let Some(prev) = app.prev_c_layers.pop() {
                    app.redo_stack.push(UndoType::Current);
                    let temp = app.current_layers.read().unwrap().clone();
                    app.next_c_layers.push(temp.clone());
                    *app.current_layers.write().unwrap() = prev;
                    populate_maps(app, temp.clone());
                    render_svg(app, ctx, temp, app.color_for_layer.values().cloned().collect());
                }
            },
        }
        info!("Undid 1 step");
    } 
}
fn redo(app: &mut SvgApp, ctx: egui::Context) {
    if let Some(undo_type) = app.redo_stack.pop() {
        match undo_type {
            UndoType::Loaded => {
                if let Some(next) = app.next_l_layers.pop(){
                    app.undo_stack.push(UndoType::Loaded);
                    app.prev_l_layers.push(app.loaded_layers.clone());
                    app.loaded_layers = next;
                    populate_maps(app, app.loaded_layers.clone());
                    render_svg(app, ctx, app.loaded_layers.clone(), app.color_for_layer.values().cloned().collect());
                }
            },
            UndoType::Current => {
                if let Some(next) = app.next_c_layers.pop(){
                    app.undo_stack.push(UndoType::Current);
                    let temp = app.current_layers.read().unwrap().clone();
                    app.prev_c_layers.push(temp.clone());
                    *app.current_layers.write().unwrap() = next;
                    populate_maps(app, temp.clone());
                    render_svg(app, ctx, temp, app.color_for_layer.values().cloned().collect());
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


fn populate_maps(app: &mut SvgApp, polylines: BTreeMap<String, Vec<PolyLine>>) {
    let mut colors = vec![
        [0.6       , 0.29803921, 0.        ],
        [1.        , 0.62745098, 0.47843137],
        [0.19607843, 0.80392157, 0.19607843],
        [0.        , 1.        , 1.        ],
        [1.        , 0.27058824, 0.        ],
        [1.        , 0.64705882, 0.        ],
        [1.        , 0.38823529, 0.27843137],
        [1.        , 0.54901961, 0.        ],
        [1.        , 0.07843137, 0.57647059],
        [0.57647059, 0.43921569, 0.85882353],
        [0.54117647, 0.16862745, 0.88627451],
        [1.        , 0.        , 0.49803921],
        [1.        , 0.75294118, 0.79607843],
        [0.23529412, 0.70196078, 0.44313725],
        [0.        , 0.98039216, 0.60392157],
        [1.        , 0.        , 0.        ],
        [1.        , 0.41176471, 0.70588235],
        [0.69019608, 0.76862745, 0.87058824],
        [0.        , 0.        , 1.        ],
        [1.        , 0.2       , 0.2       ],
        [1.        , 1.        , 0.        ],
        [0.52941176, 0.80784314, 0.92156863],
        [1.        , 1.        , 1.        ],        
        ];

    let mut checkbox_map = BTreeMap::<String, bool>::default();
    let mut color_map = BTreeMap::<String, [f32; 3]>::default();
    let mut name_map = BTreeMap::<String, String>::default();
    for layer_name in polylines.keys() {
        let color = match colors.pop() {
            Some(_color) => _color,
            None => [0., 0., 0.],
        };
        checkbox_map.insert(layer_name.clone(), true);
        color_map.insert(layer_name.clone(), color);
        name_map.insert(layer_name.clone(), layer_name.clone());
    }
    app.checkbox_for_layer = checkbox_map;
    app.color_for_layer = color_map;
    app.old_to_new_name = name_map;
}


