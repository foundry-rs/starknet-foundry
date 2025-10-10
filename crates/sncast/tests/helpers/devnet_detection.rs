use sncast::helpers::devnet::detection::detect_devnet_url;

#[tokio::test]
async fn test_detect_devnet_url() {
    let result = detect_devnet_url()
        .await
        .expect("Failed to detect devnet URL");

    assert_eq!(result, "http://127.0.0.1:5055");
}
