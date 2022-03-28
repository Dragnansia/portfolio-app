use crate::app::Response;
use eframe::{
    egui::{self, Context},
    epaint::Color32,
};
use load_image::ImageData;
use native_dialog::FileDialog;
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Formatter};
use std::{
    path::PathBuf,
    sync::{mpsc::Sender, Arc, Mutex},
};

#[derive(Deserialize, Serialize, Clone)]
pub struct Image {
    pub url: String,
    pub alt: String,

    #[serde(skip)]
    pub data: Option<egui::TextureHandle>,
    #[serde(skip)]
    pub name: String,
}

impl PartialEq for Image {
    fn eq(&self, other: &Self) -> bool {
        self.url == other.url && self.alt == other.alt
    }
}

impl Debug for Image {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "url: {}, alt: {}", self.url, self.alt)
    }
}

impl Default for Image {
    fn default() -> Self {
        Self {
            url: "/proxy-image.jpg".into(),
            alt: "No image for project".into(),
            data: None,
            name: String::new(),
        }
    }
}

impl Image {
    pub fn new() -> Self {
        Self {
            url: "".into(),
            alt: "".into(),
            ..Default::default()
        }
    }
}

fn from_path(path: &PathBuf) -> Result<egui::ColorImage, load_image::Error> {
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

pub async fn new_image_file_dialog(sender: Sender<Response>, context: Arc<Mutex<Context>>) {
    let res = FileDialog::new()
        .set_location("~")
        .add_filter("Images", &["png", "jpg", "jpeg"])
        .show_open_single_file();

    let mut image = Image::new();
    if let Ok(Some(path)) = res {
        let file = path.file_name().unwrap().to_str().unwrap();
        image.name = file.to_string();
        let data = context
            .lock()
            .unwrap()
            .load_texture(file, from_path(&path).unwrap());

        image.data = Some(data);
        sender.send(Response::Image(image)).unwrap();
    } else {
        sender.send(Response::Nothing).unwrap();
    }
}
