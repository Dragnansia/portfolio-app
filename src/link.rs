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

#[cfg(test)]
mod test {
    use super::{Link, LinkIcon};

    #[test]
    fn serialize() {
        let link = Link {
            url: "Links".into(),
            icon: LinkIcon::FontAwesome("github icon".into()),
        };

        let json = serde_json::to_string(&link).unwrap();

        println!("{:?}", json);
    }
}
