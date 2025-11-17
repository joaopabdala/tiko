use std::error::Error;

use lazy_static::lazy_static;
use regex::Regex;

#[derive(Debug, PartialEq)]
pub struct VideoInfo {
    pub username: String,
    pub video_id: String,
    pub item_type: String,
}

pub fn parse_tiktok_url(video_url: &str) -> Result<VideoInfo, Box<dyn Error>> {
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
        let item_type = captures
            .name("type")
            .map(|m| m.as_str().to_string())
            .ok_or_else(|| "Invalid URL".to_string())?;

        return Ok(VideoInfo {
            username,
            video_id,
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
        let expected = VideoInfo {
            username: String::from("user012147011"),
            video_id: String::from("7571521498322652434"),
            item_type: String::from("video"),
        };

        assert_eq!(expected, parse_tiktok_url(url).unwrap())
    }

    #[test]
    fn fails_on_invalid_url() {
        let url = "https://example.com/not/a/tiktok/url";

        assert!(parse_tiktok_url(url).is_err());
    }
}
