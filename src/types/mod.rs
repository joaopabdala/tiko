#[derive(Debug, PartialEq)]
pub enum ItemType {
    Video,
    Photo,
    Unknown,
}

impl ItemType {
    pub fn from_str(s: &str) -> Self {
        match s {
            "video" => ItemType::Video,
            "photo" => ItemType::Photo,
            _ => ItemType::Unknown,
        }
    }
}
