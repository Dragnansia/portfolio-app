use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct ImgbbResponse {
    pub data: ImgbbData,
    pub success: bool,
    pub status: i16,
}

#[derive(Deserialize, Serialize)]
pub struct ImgbbData {
    pub title: String,
    pub url: String,
    pub size: u64,
}
