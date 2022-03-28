use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Link {
    pub url: String,
    pub icon: LinkIcon,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum LinkIcon {
    Image(String),
    FontAwesome(String),
}
