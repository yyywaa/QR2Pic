use axum::{
    routing::{delete, get, post},
    Router,
};
use axum_test::TestServer;
use crate::tests::helpers::{
    delete_image, get_image, health_check, upload_image, MockImageRepository, MockStorage,
};
use uuid::Uuid;

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
async fn test_get_existing_image() {
    let (server, image_repo, _) = create_server().await;

    let image = image_repo.create("test-image.jpg".to_string()).await;
    let id = image.id.to_string();

    let response = server.get(&format!("/image/{}", id)).await;

    response.assert_status(axum::http::StatusCode::FOUND);

    let location = response.headers().get("location").unwrap();
    let url = location.to_str().unwrap();
    assert!(url.contains("test-image.jpg"));
}

#[tokio::test]
async fn test_get_nonexistent_image() {
    let (server, _, _) = create_server().await;

    let fake_id = Uuid::new_v4();
    let response = server.get(&format!("/image/{}", fake_id)).await;

    response.assert_status(axum::http::StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_get_image_invalid_uuid() {
    let (server, _, _) = create_server().await;

    let response = server.get("/image/invalid-uuid").await;

    response.assert_status(axum::http::StatusCode::BAD_REQUEST);
}
