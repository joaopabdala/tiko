use std::error::Error;

use reqwest::{Client, Url};
use serde_json::Value;

use crate::parser::TiktokInfo;

const API_URL: &str = "https://www.tikwm.com/api/";

pub async fn get_video_url(
    TiktokInfo { tiktok_id, .. }: &TiktokInfo,
) -> Result<String, Box<dyn Error>> {
    let task_id = fetch_tikwm_task_id(&tiktok_id).await?;

    let video_url = fetch_video_url(&task_id, &tiktok_id).await?;
    Ok(video_url)
}

pub async fn get_photos_url(
    TiktokInfo { tiktok_id, .. }: &TiktokInfo,
) -> Result<Vec<String>, Box<dyn Error>> {
    let url_string = format!("{}?url={}&hd=1", API_URL, tiktok_id);
    let url = Url::parse(&url_string)?;

    let client = Client::new();

    let response = client.get(url).send().await?;

    let status = response.status();
    if status.is_success() {
        let body = response.text().await?;

        let json_body: Value = serde_json::from_str(&body)?;
        check_error_code(&json_body)?;
        let images_urls_value_array: &Vec<Value> = json_body["data"]["images"]
            .as_array()
            .ok_or_else(|| -> Box<dyn Error> {
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
            .map_err(|e| -> Box<dyn Error> { e.into() })?;

        Ok(images_urls)
    } else {
        Err(format!("Error HTTP: Status {}", status).into())
    }
}

pub async fn fetch_tikwm_task_id(tiktok_id: &str) -> Result<String, Box<dyn Error>> {
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

    let status = response.status();
    if status.is_success() {
        let body = response.text().await?;

        let json_body: Value = serde_json::from_str(&body)?;

        check_error_code(&json_body)?;

        let task_id = json_body["data"]["task_id"]
            .as_str()
            .ok_or_else(|| -> Box<dyn Error> {
                format!(
                    "task_id not found in JSON response for video ID: {}, error:{}",
                    tiktok_id, json_body
                )
                .into()
            })?
            .to_string();

        Ok(task_id)
    } else {
        Err(format!("Error HTTP: Status {}", status).into())
    }
}

pub async fn fetch_video_url(task_id: &str, tiktok_id: &str) -> Result<String, Box<dyn Error>> {
    let url_string = format!("{}video/task/result?task_id={}", API_URL, task_id);
    let url = Url::parse(&url_string)?;

    let client = Client::new();
    let response = client
        .get(url)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await?;

    let status = response.status();
    if status.is_success() {
        let body = response.text().await?;

        let json_body: Value = serde_json::from_str(&body)?;
        check_error_code(&json_body)?;
        let video_url = json_body["data"]["detail"]["play_url"]
            .as_str()
            .ok_or_else(|| -> Box<dyn Error> {
                format!(
                    "video not found in JSON response for video ID: {}, error: {}",
                    tiktok_id, json_body
                )
                .into()
            })?
            .to_string();

        Ok(video_url)
    } else {
        Err(format!("Error HTTP: Status {}", status).into())
    }
}

fn check_error_code(json_body: &Value) -> Result<(), Box<dyn Error>> {
    if let Some(code) = json_body["code"].as_i64() {
        if code != 0 {
            let message = json_body["msg"]
                .as_str()
                .unwrap_or("No 'msg' field provided.")
                .to_string();
            return Err(format!("Task failed (API Code {}): {}", code, message).into());
        }
    }
    Ok(())
}
