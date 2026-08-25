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
async fn test_delete_with_correct_key() {
    let (server, image_repo, _) = create_server().await;

    let image = image_repo.create("delete-me.jpg".to_string()).await;
    let id = image.id.to_string();

    let response = server
        .delete(&format!("/delete/{}", id))
        .add_header("X-Delete-Key", "test_delete_key")
        .await;

    response.assert_status(axum::http::StatusCode::NO_CONTENT);

    let found = image_repo.find_by_id(image.id).await;
    assert!(found.is_none());
}

#[tokio::test]
async fn test_delete_with_wrong_key() {
    let (server, image_repo, _) = create_server().await;

    let image = image_repo.create("keep-me.jpg".to_string()).await;
    let id = image.id.to_string();

    let response = server
        .delete(&format!("/delete/{}", id))
        .add_header("X-Delete-Key", "wrong_key")
        .await;

    response.assert_status(axum::http::StatusCode::UNAUTHORIZED);

    let found = image_repo.find_by_id(image.id).await;
    assert!(found.is_some());
}

#[tokio::test]
async fn test_delete_without_key_header() {
    let (server, image_repo, _) = create_server().await;

    let image = image_repo.create("keep-me-2.jpg".to_string()).await;
    let id = image.id.to_string();

    let response = server.delete(&format!("/delete/{}", id)).await;

    response.assert_status(axum::http::StatusCode::UNAUTHORIZED);

    let found = image_repo.find_by_id(image.id).await;
    assert!(found.is_some());
}

#[tokio::test]
async fn test_delete_nonexistent_image() {
    let (server, _, _) = create_server().await;

    let fake_id = Uuid::new_v4();

    let response = server
        .delete(&format!("/delete/{}", fake_id))
        .add_header("X-Delete-Key", "test_delete_key")
        .await;

    response.assert_status(axum::http::StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_delete_already_deleted_image() {
    let (server, image_repo, _) = create_server().await;

    let image = image_repo.create("already-deleted.jpg".to_string()).await;
    let id = image.id.to_string();

    let _ = server
        .delete(&format!("/delete/{}", id))
        .add_header("X-Delete-Key", "test_delete_key")
        .await;

    let response = server
        .delete(&format!("/delete/{}", id))
        .add_header("X-Delete-Key", "test_delete_key")
        .await;

    response.assert_status(axum::http::StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_delete_invalid_uuid() {
    let (server, _, _) = create_server().await;

    let response = server
        .delete("/delete/not-a-uuid")
        .add_header("X-Delete-Key", "test_delete_key")
        .await;

    response.assert_status(axum::http::StatusCode::BAD_REQUEST);
}
