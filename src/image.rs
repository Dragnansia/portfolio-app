use crate::{app::Response, response::ImgbbResponse};
use const_env::from_env;
use eframe::{
    egui::{self, Context},
    epaint::Color32,
};
use load_image::ImageData;
use native_dialog::FileDialog;
use regex::Regex;
use reqwest::{
    multipart::{self, Part},
    Body, Client,
};
use rustc_serialize::{
    base64::{ToBase64, MIME},
    hex::ToHex,
};
use serde::{Deserialize, Serialize};
use std::{
    fmt::{Debug, Formatter},
    fs,
    io::Read,
};
use std::{
    path::PathBuf,
    sync::{mpsc::Sender, Arc, Mutex},
};
use tokio::fs::File;
use tokio_util::codec::{BytesCodec, FramedRead};

const IMGBB_UPLOAD_URL: &str = "https://api.imgbb.com/1/upload?key=";
#[from_env]
const API_KEY: &str = "API_KEY";

#[derive(Deserialize, Serialize, Clone)]
pub struct Image {
    pub url: String,
    pub alt: String,
    pub base64: String,

    #[serde(skip)]
    pub data: Option<egui::TextureHandle>,
    #[serde(skip)]
    pub name: String,
    #[serde(skip)]
    pub path: String,
    #[serde(skip)]
    pub is_upload: bool,
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
            is_upload: false,
            data: None,
            name: String::new(),
            path: String::new(),
            base64: String::new(),
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
        ImageData::RGBA8(p) => p
            .iter()
            .map(|el| Color32::from_rgba_premultiplied(el.r, el.g, el.b, el.a))
            .collect(),
        _ => vec![],
    };

    Ok(egui::ColorImage { size, pixels })
}

pub fn file_type(hex: &str) -> &str {
    if Regex::new(r"^ffd8ffe0").unwrap().is_match(hex) {
        return "jpeg";
    } else if Regex::new(r"^89504e47").unwrap().is_match(hex) {
        return "png";
    } else if Regex::new(r"^47494638").unwrap().is_match(hex) {
        return "gif";
    }

    panic!("Not valid type file");
}

pub fn base64_from_file(path: &PathBuf) -> String {
    let mut file = fs::File::open(path).unwrap();
    let mut vec = vec![];

    let _ = file.read_to_end(&mut vec);
    let base64 = vec.to_base64(MIME);
    let hex = vec.to_hex();

    let ext = file_type(&hex);

    return format!("data:image/{};base64,{}", ext, base64.replace("\r\n", ""));
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
        image.base64 = base64_from_file(&path);

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

pub async fn upload(sender: Sender<Response>, mut image: Image) {
    let url = format!("{}{}", IMGBB_UPLOAD_URL, API_KEY);

    let file = File::open(&image.path).await.unwrap();
    let reader = Body::wrap_stream(FramedRead::new(file, BytesCodec::new()));

    let form =
        multipart::Form::new().part("image", Part::stream(reader).file_name(image.name.clone()));

    let client = Client::new();
    let res = client.post(url).multipart(form).send().await;

    if let Ok(r) = res {
        let text = r.text().await.unwrap();
        println!("{:#?}", text);
        let res: Result<ImgbbResponse, serde_json::Error> = serde_json::from_str(&text);

        if let Ok(r) = res {
            if r.success {
                image.url = r.data.url;
                sender.send(Response::UpdateImage(image)).unwrap();
            } else {
                sender.send(Response::Nothing).unwrap();
            }
        } else {
            sender
                .send(Response::Error(res.err().unwrap().to_string()))
                .unwrap();
        }
    } else {
        sender.send(Response::Nothing).unwrap();
    }
}
