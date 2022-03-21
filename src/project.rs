use crate::image::Image;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Default, Clone)]
pub struct Project {
    pub id: u64,
    pub name: String,
    pub description: String,
    pub images: Vec<Image>,
    pub links: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, Default, Clone)]
pub struct Links {
    pub url: String,
    pub icon: String,
    pub name: String,
}

impl PartialEq for Project {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.name == other.name
    }
}
