#[cfg(test)]
mod integration_tests {
    use tiko::{
        api::{fetch_tikwm_task_id, fetch_video_url, get_video_url},
        parser::TiktokInfo,
        types::ItemType,
    };
    use tokio;

    const TEST_VIDEO_ID: &str = "7523202626222148882";
    const TEST_TASK_ID: &str = "122c687b4b562440434c1ed9ca2cfe78";

    fn create_test_video_info() -> TiktokInfo {
        TiktokInfo {
            username: String::from("testuser"),
            tiktok_id: String::from(TEST_VIDEO_ID),
            item_type: ItemType::Video,
        }
    }

    #[tokio::test]
    async fn test_get_video_url_success() {
        let expect = "https://v16.tokcdn.com/9966a86bb4ebcaa63cd07339dcdf09bd/68671980/7523202626222148882_original.mp4";
        let video_info = create_test_video_info();
        let result = get_video_url(&video_info).await.unwrap();
        assert_eq!(expect, result)
    }

    #[tokio::test]
    async fn test_fetch_task_id_success() {
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        let video_id = TEST_VIDEO_ID;
        let result = fetch_tikwm_task_id(video_id).await;

        assert!(result.is_ok(), "Failed to get Task ID: {:?}", result.err());
        let task_id = result.unwrap();
        println!("{:?}", task_id);
        // assert!(!task_id.is_empty(), "task_id empty.");
    }

    #[tokio::test]
    async fn test_full_get_video_url_success() {
        let video_info = create_test_video_info();

        let result = fetch_video_url(TEST_TASK_ID, &video_info.tiktok_id).await;
        assert!(result.is_ok(), "api requisiton failed: {:?}", result.err());

        let video_url = result.unwrap();
        assert!(video_url.starts_with("http"), "invalid url returned.");
    }
}
