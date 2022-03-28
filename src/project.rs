use crate::{image::Image, link::Link};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Default, Clone)]
pub struct Project {
    pub id: u64,
    pub name: String,
    pub description: String,
    pub images: Vec<Image>,
    pub links: Vec<Link>,
}

impl PartialEq for Project {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.name == other.name
    }
}
