pub mod api;
mod downloader;
pub mod parser;

use std::error::Error;

use api::get_video_url;
use downloader::download_video_from_url;
use parser::parse_tiktok_url;

use crate::parser::VideoInfo;

pub async fn download_from_url(video_url: &str) -> Result<(), Box<dyn Error>> {
    let video_info: VideoInfo = parse_tiktok_url(video_url)?;
    let video_url: String = get_video_url(&video_info).await?;

    download_video_from_url(video_url, &video_info).await?;
    Ok(())
}
