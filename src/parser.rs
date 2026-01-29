use std::error::Error;

use lazy_static::lazy_static;
use regex::Regex;

use crate::types::ItemType;

#[derive(Debug, PartialEq)]
pub struct TiktokInfo {
    pub username: String,
    pub tiktok_id: String,
    pub item_type: ItemType,
}

pub fn parse_tiktok_url(video_url: &str) -> Result<TiktokInfo, Box<dyn Error + Send + Sync>> {
    lazy_static! {
        static ref TIKTOK_REGEX: Regex =
            Regex::new(r"tiktok\.com/@(?P<user>[^/]+)/(?P<type>[a-z]+)/(?P<id>\d+)").unwrap();
    }

    if let Some(captures) = TIKTOK_REGEX.captures(video_url) {
        let username = captures
            .name("user")
            .map(|m| m.as_str().to_string())
            .ok_or_else(|| " Invalid URL".to_string())?;

        let video_id = captures
            .name("id")
            .map(|m| m.as_str().to_string())
            .ok_or_else(|| "Invalid URL".to_string())?;
        let item_type_str = captures
            .name("type")
            .map(|m| m.as_str().to_string())
            .ok_or_else(|| "Invalid URL".to_string())?;

        let item_type = ItemType::from_str(&item_type_str);
        return Ok(TiktokInfo {
            username,
            tiktok_id: video_id,
            item_type,
        });
    } else {
        return Err("URL format error ".into());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_parse() {
        let url = "https://www.tiktok.com/@user012147011/video/7571521498322652434";
        let expected = TiktokInfo {
            username: String::from("user012147011"),
            tiktok_id: String::from("7571521498322652434"),
            item_type: ItemType::Video,
        };

        assert_eq!(expected, parse_tiktok_url(url).unwrap())
    }

    #[test]
    fn fails_on_invalid_url() {
        let url = "https://example.com/not/a/tiktok/url";

        assert!(parse_tiktok_url(url).is_err());
    }
}
