#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::egui;


pub struct SvgApp {
    svg_image: egui_extras::RetainedImage,

}



impl Default for SvgApp {
    fn default() -> Self {
        Self {
            svg_image: egui_extras::RetainedImage::from_svg_bytes_with_size(
                "../test1_dxf_export.svg",
                include_bytes!("../test1_dxf_export.svg"),
                egui_extras::image::FitTo::Original,
            )
            .unwrap(),
        }
    }
}

impl eframe::App for SvgApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("SVG example");
            ui.label("The SVG is rasterized and displayed as a texture.");

            ui.separator();

            let max_size = ui.available_size();
            self.svg_image.show_size(ui, max_size);
        });
    }
}