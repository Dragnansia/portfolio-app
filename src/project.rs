use crate::{
    image::Image,
    link::{Link, LinkIcon},
};
use mongodb::bson::{oid::ObjectId, Bson, Document};
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

impl Project {
    pub fn new() -> Self {
        Self {
            id: Default::default(),
            name: "New Project".into(),
            description: String::new(),
            images: vec![],
            links: vec![],
        }
    }

    pub fn doc(&self) -> Document {
        let mut doc = Document::new();
        doc.insert("name", Bson::String(self.name.clone()));
        doc.insert("description", Bson::String(self.description.clone()));

        doc.insert(
            "images",
            Bson::Array(
                self.images
                    .iter()
                    .map(|i| {
                        let mut doc = Document::new();
                        doc.insert("name", Bson::String(i.name.clone()));
                        doc.insert("data", Bson::String(i.data.clone()));

                        Bson::Document(doc)
                    })
                    .collect(),
            ),
        );

        doc.insert(
            "links",
            Bson::Array(
                self.links
                    .iter()
                    .map(|i| {
                        let mut doc = Document::new();
                        doc.insert("url", Bson::String(i.url.clone()));

                        let val = match &i.icon {
                            LinkIcon::Image(i) => {
                                let mut d = Document::new();
                                d.insert("Image", Bson::String(i.clone()));
                                d
                            }
                            LinkIcon::FontAwesome(i) => {
                                let mut d = Document::new();
                                d.insert("FontAwesome", Bson::String(i.clone()));
                                d
                            }
                        };

                        doc.insert("icon", Bson::Document(val));

                        Bson::Document(doc)
                    })
                    .collect(),
            ),
        );

        doc
    }
}
