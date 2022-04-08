use crate::app::Response;
use eframe::egui::{self, Context};
use native_dialog::FileDialog;
use rustc_serialize::base64::{ToBase64, MIME};
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Formatter};
use std::{
    path::{Path, PathBuf},
    sync::{mpsc::Sender, Arc, Mutex},
};

#[derive(Deserialize, Serialize, Clone)]
pub struct Image {
    pub alt: String,
    pub data: String,

    #[serde(skip)]
    pub egui_data: Option<egui::TextureHandle>,
    #[serde(skip)]
    pub name: String,
    #[serde(skip)]
    pub path: String,
    #[serde(skip)]
    pub is_upload: bool,
}

impl PartialEq for Image {
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data && self.alt == other.alt
    }
}

impl Debug for Image {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "data: {}, alt: {}", self.data, self.alt)
    }
}

impl Default for Image {
    fn default() -> Self {
        Self {
            data: "/proxy-image.jpg".into(),
            alt: "No image for project".into(),
            is_upload: false,
            egui_data: None,
            name: String::new(),
            path: String::new(),
        }
    }
}

impl Image {
    pub fn new() -> Self {
        Self {
            data: "".into(),
            alt: "".into(),
            ..Default::default()
        }
    }
}

fn from_path(path: &PathBuf) -> Result<(Vec<u8>, egui::ColorImage), image::ImageError> {
    let image = image::io::Reader::open(&path)?.decode()?;
    let size = [image.width() as _, image.height() as _];

    let image_buffer = image.to_rgba8();
    let flat_samples = image_buffer.as_flat_samples();
    let pixels = flat_samples.as_slice();

    Ok((
        pixels.to_vec(),
        egui::ColorImage::from_rgba_unmultiplied(size, pixels),
    ))
}

pub fn file_type(extension: &str) -> &str {
    match extension {
        "jpg" | "jpeg" => "jpeg",
        "png" => "png",
        _ => "",
    }
}

pub fn base64_from_vec(vec: Vec<u8>, path: &Path) -> String {
    let base64 = vec.to_base64(MIME);

    let ext = file_type(path.extension().unwrap().to_str().unwrap());
    let base64 = base64.replace("\r\n", "");
    return format!("data:image/{};base64,{}", ext, base64);
}

pub async fn new_image_file_dialog(sender: Sender<Response>, context: Arc<Mutex<Context>>) {
    let res = FileDialog::new()
        .set_location("~")
        .add_filter("Images", &["png", "jpg", "jpeg"])
        .show_open_single_file();

    let mut image = Image::new();
    if let Ok(Some(path)) = res {
        let file = path.file_name().unwrap().to_str().unwrap();

        image.path = path.to_str().unwrap().to_string();
        image.name = file.to_string();

        let (base64, data) = from_path(&path).unwrap();
        image.data = base64_from_vec(base64, &path);

        let data = context.lock().unwrap().load_texture(file, data);

        image.egui_data = Some(data);
        sender.send(Response::Image(image)).unwrap();
    } else {
        sender.send(Response::Nothing).unwrap();
    }
}
