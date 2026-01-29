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

    let mut attempts = 0;
    let max_retries = 3;

    let result = loop {
        let permit = sem.acquire().await.unwrap();

        let outcome = match tiktok_info.item_type {
            ItemType::Video => get_video_url(&tiktok_info).await.map(DownloadTarget::Video),
            ItemType::Photo => get_photos_url(&tiktok_info)
                .await
                .map(DownloadTarget::Photos),
            ItemType::Unknown => {
                drop(permit);
                return Ok(());
            }
        };

        match outcome {
            Ok(target) => {
                sleep(Duration::from_secs(1)).await;
                drop(permit);
                break target;
            }
            Err(e) if attempts < max_retries => {
                attempts += 1;
                eprintln!(
                    "Attempt {}/{} failed: {} for {}. Trying again in 2s...",
                    attempts, max_retries, tiktok_url, e
                );
                drop(permit);
                sleep(Duration::from_secs(2)).await;
            }
            Err(e) => return Err(e),
        }
    };

    match result {
        DownloadTarget::Video(url) => download_video_from_url(url, &tiktok_info).await?,
        DownloadTarget::Photos(urls) => download_photos_from_url(urls, &tiktok_info).await?,
    }

    Ok(())
}

enum DownloadTarget {
    Video(String),
    Photos(Vec<String>),
}
