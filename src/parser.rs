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

/// Splits input that may hold several URLs pasted together (no separators) into one slice per URL.
/// Input with no recognizable TikTok URL is returned as-is so `parse_tiktok_id` reports the error.
pub fn split_tiktok_urls(input: &str) -> Vec<&str> {
    lazy_static! {
        static ref TIKTOK_URL_REGEX: Regex =
            Regex::new(r"(?:https?://)?(?:www\.)?tiktok\.com/@[^/\s]+/[a-z]+/\d+").unwrap();
    }

    let urls: Vec<&str> = TIKTOK_URL_REGEX
        .find_iter(input)
        .map(|m| m.as_str())
        .collect();

    if urls.is_empty() { vec![input] } else { urls }
}

/// Accepts a bare ID or a TikTok URL. Only the ID is taken from a URL: the username and
/// item type in it may be wrong, so they are resolved via the API instead.
pub fn parse_tiktok_id(input: &str) -> Result<String, Box<dyn Error + Send + Sync>> {
    lazy_static! {
        static ref TIKTOK_REGEX: Regex =
            Regex::new(r"tiktok\.com/@[^/]+/[a-z]+/(?P<id>\d+)").unwrap();
    }

    let input = input.trim();
    if !input.is_empty() && input.chars().all(|c| c.is_ascii_digit()) {
        return Ok(input.to_string());
    }

    TIKTOK_REGEX
        .captures(input)
        .and_then(|captures| captures.name("id"))
        .map(|m| m.as_str().to_string())
        .ok_or_else(|| format!("URL format error: {}", input).into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_id_from_url() {
        let url = "https://www.tiktok.com/@some_user/video/4444444444444444444?lang=pt";

        assert_eq!("4444444444444444444", parse_tiktok_id(url).unwrap())
    }

    #[test]
    fn splits_concatenated_urls() {
        let input = "https://www.tiktok.com/@user_a/video/1111111111111111111https://www.tiktok.com/@user_b/video/2222222222222222222https://www.tiktok.com/@user_c/photo/3333333333333333333";

        assert_eq!(
            split_tiktok_urls(input),
            vec![
                "https://www.tiktok.com/@user_a/video/1111111111111111111",
                "https://www.tiktok.com/@user_b/video/2222222222222222222",
                "https://www.tiktok.com/@user_c/photo/3333333333333333333",
            ]
        );
    }

    #[test]
    fn splits_concatenated_urls_with_query_strings() {
        let input =
            "https://www.tiktok.com/@a/video/111?lang=pthttps://www.tiktok.com/@b/video/222";

        assert_eq!(
            split_tiktok_urls(input),
            vec![
                "https://www.tiktok.com/@a/video/111",
                "https://www.tiktok.com/@b/video/222",
            ]
        );
    }

    #[test]
    fn split_keeps_single_url_and_invalid_input() {
        let url = "https://www.tiktok.com/@some_user/video/4444444444444444444";
        assert_eq!(split_tiktok_urls(url), vec![url]);

        let invalid = "https://example.com/not/a/tiktok/url";
        assert_eq!(split_tiktok_urls(invalid), vec![invalid]);
    }

    #[test]
    fn parses_bare_id() {
        assert_eq!(
            "1111111111111111111",
            parse_tiktok_id("1111111111111111111").unwrap()
        );
    }

    #[test]
    fn fails_on_invalid_input() {
        assert!(parse_tiktok_id("").is_err());
        assert!(parse_tiktok_id("123abc").is_err());
    }

    #[test]
    fn fails_on_invalid_url() {
        let url = "https://example.com/not/a/tiktok/url";

        assert!(parse_tiktok_id(url).is_err());
    }
}
