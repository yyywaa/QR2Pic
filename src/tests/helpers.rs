use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use chrono::{DateTime, Utc};
use serde_json::{json, Value};
use std::sync::Arc;
use std::path::PathBuf;
use tokio::sync::Mutex;
use uuid::Uuid;

#[derive(Clone)]
pub struct MockStorage {
    pub uploads: Arc<Mutex<Vec<UploadedFile>>>,
    pub base_path: PathBuf,
    pub base_url: String,
}

#[derive(Clone)]
pub struct MockImageRepository {
    pub images: Arc<Mutex<Vec<MockImage>>>,
}

#[derive(Debug, Clone)]
pub struct UploadedFile {
    pub key: String,
    pub content: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct MockImage {
    pub id: Uuid,
    pub file_path: String,
    pub created_at: DateTime<Utc>,
}

impl MockStorage {
    pub fn new() -> Self {
        Self {
            uploads: Arc::new(Mutex::new(Vec::new())),
            base_path: PathBuf::from("/tmp/test_uploads"),
            base_url: "http://localhost:3000/images".to_string(),
        }
    }

    pub async fn save_file(
        &self,
        key: &str,
        content: Vec<u8>,
    ) -> Result<String, String> {
        let uploads = &mut *self.uploads.lock().await;
        uploads.push(UploadedFile {
            key: key.to_string(),
            content,
        });
        Ok(key.to_string())
    }

    pub async fn delete_file(&self, _key: &str) -> Result<(), String> {
        Ok(())
    }

    pub fn get_file_url(&self, key: &str) -> String {
        format!("{}/{}", self.base_url, key)
    }

    pub fn get_file_path(&self, key: &str) -> PathBuf {
        self.base_path.join(key)
    }

    pub fn get_file_extension(filename: &str) -> Option<String> {
        std::path::Path::new(filename)
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_lowercase())
    }

    pub fn is_allowed_extension(extension: &str) -> bool {
        matches!(extension, "jpg" | "jpeg" | "png" | "gif" | "webp")
    }
}

impl MockImageRepository {
    pub fn new() -> Self {
        Self {
            images: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub async fn create(&self, file_path: String) -> MockImage {
        let image = MockImage {
            id: Uuid::new_v4(),
            file_path,
            created_at: Utc::now(),
        };
        let images = &mut *self.images.lock().await;
        images.push(image.clone());
        image
    }

    pub async fn find_by_id(&self, id: Uuid) -> Option<MockImage> {
        let images = self.images.lock().await;
        images.iter().find(|img| img.id == id).cloned()
    }

    pub async fn delete_by_id(&self, id: Uuid) -> bool {
        let images = &mut *self.images.lock().await;
        let len_before = images.len();
        images.retain(|img| img.id != id);
        images.len() < len_before
    }
}

pub type TestAppState = (
    MockImageRepository,
    MockStorage,
    String,
);

pub async fn health_check() -> &'static str {
    "OK"
}

pub async fn upload_image(
    State((image_repo, storage, _delete_key)): State<TestAppState>,
    mut multipart: axum::extract::Multipart,
) -> Response {
    let mut file_data = None;
    let mut file_name = None;

    while let Ok(Some(field)) = multipart.next_field().await.map_err(|e| e.to_string()) {
        let field_name = field.name().unwrap_or("").to_string();

        if field_name == "file" {
            let name = field.file_name().map(|s: &str| s.to_string());
            if let Ok(data) = field.bytes().await {
                if data.len() > 10 * 1024 * 1024 {
                    return (StatusCode::PAYLOAD_TOO_LARGE, "File too large").into_response();
                }
                file_data = Some(data.to_vec());
                file_name = name;
                break;
            }
        }
    }

    let file_data = match file_data {
        Some(d) => d,
        None => return (StatusCode::BAD_REQUEST, "Missing file").into_response(),
    };

    let file_name = match file_name {
        Some(n) => n,
        None => return (StatusCode::BAD_REQUEST, "Missing file name").into_response(),
    };

    let extension = match MockStorage::get_file_extension(&file_name) {
        Some(ext) => ext,
        None => return (StatusCode::BAD_REQUEST, "Invalid file type").into_response(),
    };

    if !MockStorage::is_allowed_extension(&extension) {
        return (StatusCode::BAD_REQUEST, "Invalid file type").into_response();
    }

    let id = Uuid::new_v4();
    let key = format!("{}.{}", id, extension);

    if storage.save_file(&key, file_data).await.is_err() {
        return (StatusCode::INTERNAL_SERVER_ERROR, "Upload failed").into_response();
    }

    let file_url = storage.get_file_url(&key);

    let image = image_repo.create(key).await;

    (StatusCode::OK, Json(json!({
        "id": image.id,
        "url": file_url
    }))).into_response()
}

pub async fn get_image(
    State((image_repo, storage, _delete_key)): State<TestAppState>,
    Path(id): Path<Uuid>,
) -> Response {
    let image = image_repo.find_by_id(id).await;

    match image {
        Some(img) => {
            let url = storage.get_file_url(&img.file_path);
            let mut headers = HeaderMap::new();
            headers.insert(
                axum::http::header::LOCATION,
                url.parse().unwrap(),
            );
            (StatusCode::FOUND, headers).into_response()
        }
        None => (StatusCode::NOT_FOUND, "Image not found").into_response(),
    }
}

pub async fn delete_image(
    State((image_repo, _storage, delete_key)): State<TestAppState>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
) -> Response {
    let provided_key = match headers.get("X-Delete-Key").and_then(|v| v.to_str().ok()) {
        Some(key) => key,
        None => return (StatusCode::UNAUTHORIZED, "Missing delete key").into_response(),
    };

    if provided_key != delete_key {
        return (StatusCode::UNAUTHORIZED, "Invalid delete key").into_response();
    }

    let deleted = image_repo.delete_by_id(id).await;

    if deleted {
        (StatusCode::NO_CONTENT, "").into_response()
    } else {
        (StatusCode::NOT_FOUND, "Image not found").into_response()
    }
}
