use std::error::Error;

use reqwest::{Client, Url};
use serde_json::Value;

use crate::parser::VideoInfo;

const API_URL: &str = "https://www.tikwm.com/api/";

pub async fn get_video_url(
    VideoInfo { video_id, .. }: &VideoInfo,
) -> Result<String, Box<dyn Error>> {
    let task_id = fetch_tikwm_task_id(&video_id).await?;

    let video_url = fetch_video_url(&task_id, &video_id).await?;
    Ok(video_url)
}

pub async fn fetch_tikwm_task_id(video_id: &str) -> Result<String, Box<dyn Error>> {
    let url_string = format!("{}/video/task/submit", API_URL);
    let url = Url::parse(&url_string)?;

    let client = Client::new();

    let form_data = [("url", video_id), ("web", "1")];

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
                    video_id, json_body
                )
                .into()
            })?
            .to_string();

        Ok(task_id)
    } else {
        Err(format!("Error HTTP: Status {}", status).into())
    }
}

pub async fn fetch_video_url(task_id: &str, video_id: &str) -> Result<String, Box<dyn Error>> {
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
                    video_id, json_body
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
