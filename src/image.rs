use crate::app::Response;
use eframe::{
    egui::{self, Context},
    epaint::Color32,
};
use load_image::ImageData;
use native_dialog::FileDialog;
use std::sync::{mpsc::Sender, Arc, Mutex};

pub fn from_path(path: &std::path::Path) -> Result<egui::ColorImage, load_image::Error> {
    let img = load_image::load_path(path)?;
    let size = [img.width, img.height];
    let pixels: Vec<Color32> = match img.bitmap {
        ImageData::RGB8(p) => p
            .iter()
            .map(|el| Color32::from_rgb(el.r, el.g, el.b))
            .collect(),
        load_image::ImageData::RGBA8(p) => p
            .iter()
            .map(|el| Color32::from_rgba_premultiplied(el.r, el.g, el.b, el.a))
            .collect(),
        _ => vec![],
    };

    Ok(egui::ColorImage { size, pixels })
}

pub fn new_image_file_dialog(sender: Sender<Response>, context: Arc<Mutex<Context>>) {
    let res = FileDialog::new()
        .set_location("~")
        .add_filter("Images", &["png", "jpg", "jpeg"])
        .show_open_single_file();

    if let Ok(Some(path)) = res {
        let file = path.file_name().unwrap().to_str().unwrap();

        sender
            .send(Response::Image(
                context
                    .lock()
                    .unwrap()
                    .load_texture(file, from_path(&path).unwrap()),
            ))
            .unwrap();
    } else {
        sender.send(Response::Nothing).unwrap();
    }
}
