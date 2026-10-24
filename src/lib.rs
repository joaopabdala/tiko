pub mod api;
mod downloader;
pub mod parser;
pub mod types;

use std::{error::Error, sync::Arc, time::Duration};

use api::get_video_url;
use downloader::download_video_from_url;
use parser::parse_tiktok_id;
use tokio::{sync::Semaphore, time::sleep};

use crate::{
    api::{get_photos_url, get_video_url_fallback, resolve_tiktok_info},
    downloader::download_photos_from_url,
    parser::TiktokInfo,
    types::ItemType,
};

pub async fn download_from_url(
    tiktok_url: &str,
    sem: Arc<Semaphore>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let tiktok_id = parse_tiktok_id(tiktok_url)?;
    let tiktok_info: TiktokInfo = {
        let _permit = sem.acquire().await.unwrap();
        let info = resolve_tiktok_info(&tiktok_id).await?;
        sleep(Duration::from_secs(1)).await;
        info
    };

    let mut attempts = 0;
    let max_retries = 3;

    let result = loop {
        let permit = sem.acquire().await.unwrap();

        let outcome = match tiktok_info.item_type {
            ItemType::Video => get_video_url(&tiktok_info)
                .await
                .map(|url| DownloadTarget::Video {
                    url,
                    low_quality: false,
                }),
            ItemType::Photo => get_photos_url(&tiktok_info)
                .await
                .map(DownloadTarget::Photos),
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
            Err(e) if tiktok_info.item_type == ItemType::Video => {
                eprintln!(
                    "HD download failed for {}: {}. Falling back to lower quality...",
                    tiktok_url, e
                );
                let url = get_video_url_fallback(&tiktok_info).await;
                drop(permit);
                break DownloadTarget::Video {
                    url: url?,
                    low_quality: true,
                };
            }
            Err(e) => return Err(e),
        }
    };

    match result {
        DownloadTarget::Video { url, low_quality } => {
            download_video_from_url(url, &tiktok_info, low_quality).await?
        }
        DownloadTarget::Photos(urls) => download_photos_from_url(urls, &tiktok_info).await?,
    }

    Ok(())
}

enum DownloadTarget {
    Video { url: String, low_quality: bool },
    Photos(Vec<String>),
}
