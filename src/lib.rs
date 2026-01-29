pub mod api;
mod downloader;
pub mod parser;
pub mod types;

use std::{error::Error, sync::Arc, time::Duration};

use api::get_video_url;
use downloader::download_video_from_url;
use parser::parse_tiktok_url;
use tokio::{sync::Semaphore, time::sleep};

use crate::{
    api::get_photos_url, downloader::download_photos_from_url, parser::TiktokInfo, types::ItemType,
};

pub async fn download_from_url(
    tiktok_url: &str,
    sem: Arc<Semaphore>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let tiktok_info: TiktokInfo = parse_tiktok_url(tiktok_url)?;

    let permit = sem.acquire().await.unwrap();

    match tiktok_info.item_type {
        ItemType::Video => {
            let video_url = get_video_url(&tiktok_info).await?;
            sleep(Duration::from_secs(1)).await;
            drop(permit);

            download_video_from_url(video_url, &tiktok_info).await?;
        }
        ItemType::Photo => {
            let photos_urls = get_photos_url(&tiktok_info).await?;
            sleep(Duration::from_secs(1)).await;
            drop(permit);

            download_photos_from_url(photos_urls, &tiktok_info).await?;
        }
        ItemType::Unknown => {
            drop(permit);
            eprintln!("Tipo desconhecido.");
        }
    }

    Ok(())
}
