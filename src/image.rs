use eframe::{egui, epaint::Color32};
use load_image::ImageData;

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
