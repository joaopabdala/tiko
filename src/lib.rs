pub mod api;
mod downloader;
pub mod parser;
pub mod types;

use std::error::Error;

use api::get_video_url;
use downloader::download_video_from_url;
use parser::parse_tiktok_url;

use crate::{
    api::get_photos_url, downloader::download_photos_from_url, parser::TiktokInfo, types::ItemType,
};

pub async fn download_from_url(tiktok_url: &str) -> Result<(), Box<dyn Error>> {
    let tiktok_info: TiktokInfo = parse_tiktok_url(tiktok_url)?;

    match tiktok_info.item_type {
        ItemType::Video => {
            let tiktok_url: String = get_video_url(&tiktok_info).await?;
            download_video_from_url(tiktok_url, &tiktok_info).await?;
            Ok(())
        }
        ItemType::Photo => {
            let photos_urls = get_photos_url(&tiktok_info).await?;
            download_photos_from_url(photos_urls, &tiktok_info).await?;
            Ok(())
        }
        ItemType::Unknown => {
            eprintln!("Tipo de item desconhecido, pulando download.");
            Ok(())
        }
    }
}
