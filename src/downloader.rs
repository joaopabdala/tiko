use std::error::Error;

use futures_util::stream::StreamExt;
use reqwest::Client;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

use crate::parser::VideoInfo;

pub async fn download_video_from_url(
    video_url: String,
    video_info: &VideoInfo,
) -> Result<(), Box<dyn Error>> {
    let client = Client::new();

    let response = client.get(&video_url).send().await?;

    let status = response.status();
    if !status.is_success() {
        return Err(format!("Failed to download video. HTTP status: {}", status).into());
    }
    eprintln!(
        "download started for {}_{}",
        video_info.username, video_info.video_id
    );
    let filename = format!("{}_{}.mp4", video_info.username, video_info.video_id);
    let mut file = File::create(&filename).await?;
    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream.next().await {
        file.write_all(&chunk?).await?;
    }

    println!("download finished '{}'.", filename);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::VideoInfo;
    use tokio::io::AsyncReadExt;

    fn create_test_info() -> VideoInfo {
        VideoInfo {
            username: String::from("test_user"),
            video_id: String::from("12345"),
            item_type: String::from("video"),
        }
    }

    #[tokio::test]
    async fn test_download_video_success() {
        let test_data = b"This is the fake video content";
        let video_info = create_test_info();
        let expected_filename = format!("{}_{}.mp4", video_info.username, video_info.video_id);

        let mut server = mockito::Server::new_async().await;

        let mock = server
            .mock("GET", "/video.mp4")
            .with_status(200)
            .with_body(test_data)
            .create();

        let mock_url = format!("{}/video.mp4", server.url());

        let result = download_video_from_url(mock_url, &video_info).await;

        assert!(result.is_ok(),);

        mock.assert();

        let mut downloaded_file = tokio::fs::File::open(&expected_filename).await.unwrap();
        let mut contents = Vec::new();
        downloaded_file.read_to_end(&mut contents).await.unwrap();

        assert_eq!(contents, test_data.to_vec(),);

        tokio::fs::remove_file(expected_filename).await.unwrap();
    }

    #[tokio::test]
    async fn test_download_video_http_fail() {
        let test_data = b"This is the fake video content";
        let video_info = create_test_info();

        let mut server = mockito::Server::new_async().await;

        let mock = server
            .mock("GET", "/video.mp4")
            .with_status(404)
            .with_body(test_data)
            .create();

        let mock_url = format!("{}/video.mp4", server.url());

        let result = download_video_from_url(mock_url, &video_info).await;

        assert!(result.is_err());

        let error_message = format!("{}", result.unwrap_err());
        assert!(error_message.contains("404"),);

        mock.assert();
    }
}
