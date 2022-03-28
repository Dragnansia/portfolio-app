use crate::{image::Image, link::Link};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Project {
    #[serde(skip_serializing, rename = "_id")]
    pub id: ObjectId,
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
