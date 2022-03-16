use eframe::{egui, epi, run_native, NativeOptions};

struct Portfolio;

impl epi::App for Portfolio {
    fn update(&mut self, _: &egui::Context, _: &epi::Frame) {}

    fn name(&self) -> &str {
        "Portfolio App"
    }
}

fn main() {
    run_native(Box::new(Portfolio), NativeOptions::default());
}
