mod image;

use eframe::{
    egui::{self, Layout, Window},
    epi, run_native, NativeOptions,
};
use native_dialog::FileDialog;

struct Portfolio {
    image: Vec<egui::TextureHandle>,
}

impl epi::App for Portfolio {
    fn update(&mut self, ctx: &egui::Context, _: &epi::Frame) {
        Window::new("Images").show(ctx, |ui| {
            let layout = Layout::top_down(eframe::emath::Align::Center);
            ui.with_layout(layout, |ui| {
                if ui.button("Add Image").clicked() {
                    // Do this on other thread
                    let res = FileDialog::new()
                        .set_location("~")
                        .add_filter("PNG Image", &["png"])
                        .add_filter("JPG Image", &["jpg", "jpeg"])
                        .show_open_single_file();

                    if let Ok(Some(path)) = res {
                        let file = path.file_name().unwrap().to_str().unwrap();

                        self.image.push(
                            ui.ctx()
                                .load_texture(file, image::from_path(&path).unwrap()),
                        );
                    }
                }

                for image in &self.image {
                    let img_size = 160.0 * image.size_vec2() / image.size_vec2().y;
                    ui.image(image, img_size);
                }
            });
        });
    }

    fn name(&self) -> &str {
        "Portfolio App"
    }
}

fn main() {
    run_native(
        Box::new(Portfolio { image: vec![] }),
        NativeOptions::default(),
    );
}
