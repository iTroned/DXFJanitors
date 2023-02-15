#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::{egui, glow::{FILL, BLUE}};
use egui_extras::image::FitTo;
use egui::Color32;


pub struct SvgApp {
    svg_image: egui_extras::RetainedImage,

}

impl Default for SvgApp {
    fn default() -> Self {
        Self {
            svg_image: egui_extras::RetainedImage::from_svg_bytes_with_size(
                "../test_dxf_export.svg", //path of svg file to display
                include_bytes!("../test_dxf_export.svg"), 
                FitTo::Size(3840, 2160), //display resolution (need to check performance effect)
            )
            .unwrap(),
        }
    }
}

//design of the app, look at documenation for inspiration
impl eframe::App for SvgApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        //design the frame
        let my_frame = egui::containers::Frame {
            inner_margin: egui::style::Margin { left: 10., right: 0., top: 10., bottom: 10. }, //margins (affects the color-border)
            outer_margin: egui::style::Margin { left: 0., right: 0., top: 0., bottom: 0. },
            rounding: egui::Rounding { nw: 1.0, ne: 1.0, sw: 1.0, se: 1.0 },
            shadow: eframe::epaint::Shadow { extrusion: 1.0, color: Color32::YELLOW },
            fill: Color32::from_rgb(250,249,246), //background fill color, affected by the margin
            stroke: egui::Stroke::new(2.0, Color32::BLACK),
        };
        //ui panels
        egui::CentralPanel::default().frame(my_frame).show(ctx, |ui| {
            ui.heading("SVG example");
            ui.label("The SVG is rasterized and displayed as a texture.");

            ui.separator();
            

            let max_size = ui.available_size();
            self.svg_image.show_size(ui, max_size);
        });
    }
}