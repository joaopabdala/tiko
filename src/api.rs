use std::error::Error;

use reqwest::{Client, Response, Url};
use serde_json::Value;

use crate::{parser::TiktokInfo, types::ItemType};

const API_URL: &str = "https://www.tikwm.com/api/";

pub async fn get_video_url(
    TiktokInfo { tiktok_id, .. }: &TiktokInfo,
) -> Result<String, Box<dyn Error + Send + Sync>> {
    let task_id = fetch_tikwm_task_id(&tiktok_id).await?;

    let video_url = fetch_video_url(&task_id, &tiktok_id).await?;
    Ok(video_url)
}

pub async fn get_photos_url(
    TiktokInfo { tiktok_id, .. }: &TiktokInfo,
) -> Result<Vec<String>, Box<dyn Error + Send + Sync>> {
    let json_body = fetch_item_info(tiktok_id).await?;
    let images_urls_value_array: &Vec<Value> =
        json_body["data"]["images"]
            .as_array()
            .ok_or_else(|| -> Box<dyn Error + Send + Sync> {
                format!(
                    "Images array not found in JSON response for ID: {}, error: {}",
                    tiktok_id, json_body
                )
                .into()
            })?;

    let images_urls: Vec<String> = images_urls_value_array
        .iter()
        .map(|v| {
            v.as_str()
                .map(|s| s.to_string())
                .ok_or_else(|| "URL not found in array.".to_string())
        })
        .collect::<Result<Vec<String>, String>>()
        .map_err(|e| -> Box<dyn Error + Send + Sync> { e.into() })?;

    Ok(images_urls)
}

/// Lower quality than `get_video_url`; only meant as a fallback when the task flow fails
/// (e.g. tikwm rejects stories on `video/task/submit`).
pub async fn get_video_url_fallback(
    TiktokInfo { tiktok_id, .. }: &TiktokInfo,
) -> Result<String, Box<dyn Error + Send + Sync>> {
    let json_body = fetch_item_info(tiktok_id).await?;
    ["hdplay", "play"]
        .iter()
        .filter_map(|key| json_body["data"][key].as_str())
        .find(|url| !url.is_empty())
        .map(|url| url.to_string())
        .ok_or_else(|| -> Box<dyn Error + Send + Sync> {
            format!(
                "video not found in fallback JSON response for video ID: {}, error: {}",
                tiktok_id, json_body
            )
            .into()
        })
}

/// Builds `TiktokInfo` from a bare ID, since only the API knows its username and item type.
pub async fn resolve_tiktok_info(
    tiktok_id: &str,
) -> Result<TiktokInfo, Box<dyn Error + Send + Sync>> {
    let json_body = fetch_item_info(tiktok_id).await?;
    let username = json_body["data"]["author"]["unique_id"]
        .as_str()
        .ok_or_else(|| -> Box<dyn Error + Send + Sync> {
            format!(
                "author not found in JSON response for ID: {}, error: {}",
                tiktok_id, json_body
            )
            .into()
        })?
        .to_string();
    let item_type = if json_body["data"]["images"].is_array() {
        ItemType::Photo
    } else {
        ItemType::Video
    };

    Ok(TiktokInfo {
        username,
        tiktok_id: tiktok_id.to_string(),
        item_type,
    })
}

async fn fetch_item_info(tiktok_id: &str) -> Result<Value, Box<dyn Error + Send + Sync>> {
    let url_string = format!("{}?url={}&hd=1", API_URL, tiktok_id);
    let url = Url::parse(&url_string)?;

    let client = Client::new();

    let response = client.get(url).send().await?;

    parse_response(response, "item info").await
}

pub async fn fetch_tikwm_task_id(tiktok_id: &str) -> Result<String, Box<dyn Error + Send + Sync>> {
    let url_string = format!("{}/video/task/submit", API_URL);
    let url = Url::parse(&url_string)?;

    let client = Client::new();

    let form_data = [("url", tiktok_id), ("web", "1")];

    let response = client
        .post(url)
        .form(&form_data)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await?;

    let json_body = parse_response(response, "task submit").await?;

    let task_id = json_body["data"]["task_id"]
        .as_str()
        .ok_or_else(|| -> Box<dyn Error + Send + Sync> {
            format!(
                "task_id not found in JSON response for video ID: {}, error:{}",
                tiktok_id, json_body
            )
            .into()
        })?
        .to_string();

    Ok(task_id)
}

pub async fn fetch_video_url(
    task_id: &str,
    tiktok_id: &str,
) -> Result<String, Box<dyn Error + Send + Sync>> {
    let url_string = format!("{}video/task/result?task_id={}", API_URL, task_id);
    let url = Url::parse(&url_string)?;

    let client = Client::new();
    let response = client
        .get(url)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await?;

    let json_body = parse_response(response, "task result").await?;
    let video_url = json_body["data"]["detail"]["play_url"]
        .as_str()
        .ok_or_else(|| -> Box<dyn Error + Send + Sync> {
            format!(
                "video not found in JSON response for video ID: {}, error: {}",
                tiktok_id, json_body
            )
            .into()
        })?
        .to_string();

    Ok(video_url)
}

/// Reads a tikwm response, naming the failed `step` and including the response body in errors.
async fn parse_response(
    response: Response,
    step: &str,
) -> Result<Value, Box<dyn Error + Send + Sync>> {
    let status = response.status();
    let body = response.text().await?;

    if !status.is_success() {
        return Err(format!("tikwm {} failed: HTTP {}, body: {}", step, status, body).into());
    }

    let json_body: Value = serde_json::from_str(&body).map_err(|e| {
        format!(
            "tikwm {} failed: invalid JSON ({}), body: {}",
            step, e, body
        )
    })?;
    check_error_code(&json_body, step)?;
    Ok(json_body)
}

fn check_error_code(json_body: &Value, step: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
    if let Some(code) = json_body["code"].as_i64() {
        if code != 0 {
            let message = json_body["msg"]
                .as_str()
                .unwrap_or("No 'msg' field provided.")
                .to_string();
            return Err(format!("tikwm {} failed (API Code {}): {}", step, code, message).into());
        }
    }
    Ok(())
}
