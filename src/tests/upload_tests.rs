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
async fn test_upload_rejects_empty_body() {
    let (server, image_repo, storage) = create_server().await;

    let response = server.post("/upload").await;

    response.assert_status(axum::http::StatusCode::BAD_REQUEST);

    assert_eq!(image_repo.images.lock().await.len(), 0);
    assert_eq!(storage.uploads.lock().await.len(), 0);
}
