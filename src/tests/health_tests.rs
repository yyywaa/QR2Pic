use axum::{
    routing::{delete, get, post},
    Router,
};
use axum_test::TestServer;
use crate::tests::helpers::{
    delete_image, get_image, health_check, upload_image, MockImageRepository, MockStorage,
};

async fn create_server() -> (TestServer, MockImageRepository, MockStorage) {
    let image_repo = MockImageRepository::new();
    let storage = MockStorage::new();
    let delete_key = "test_delete_key".to_string();

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/upload", post(upload_image))
        .route("/image/:id", get(get_image))
        .route("/delete/:id", delete(delete_image))
        .with_state((image_repo.clone(), storage.clone(), delete_key));

    let server = TestServer::new(app).unwrap();
    (server, image_repo, storage)
}

#[tokio::test]
async fn test_health_check() {
    let (server, _, _) = create_server().await;

    let response = server.get("/health").await;
    response.assert_status_ok();
    assert_eq!(response.text(), "OK");
}

#[tokio::test]
async fn test_health_check_multiple_calls() {
    let (server, _, _) = create_server().await;

    for _ in 0..5 {
        let response = server.get("/health").await;
        response.assert_status_ok();
        assert_eq!(response.text(), "OK");
    }
}
