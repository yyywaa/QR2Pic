use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Response,
};
use crate::error::AppError;
use uuid::Uuid;

type AppState = (
    sqlx::PgPool,
    crate::storage::Storage,
    crate::db::ImageRepository,
    String,
);

pub async fn get_view(
    State((_, storage, image_repo, _)): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Response, AppError> {
    let image = image_repo
        .find_by_id(id)
        .await?
        .ok_or(AppError::ImageNotFound)?;

    let file_path = storage.get_file_path(&image.file_path);
    
    if !file_path.exists() {
        return Err(AppError::ImageNotFound);
    }

let image_data_url = format!("/view-data/{}", id);
    let html_content = format!(
        r#"<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <title>标志明码查询</title>
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <style>
        html, body {{
            margin: 0;
            padding: 0;
            width: 100%;
            height: 100%;
            overflow: hidden;
        }}
        img {{
            width: 100%;
            height: 100%;
            object-fit: contain;
        }}
    </style>
</head>
<body>
    <img src="{}" alt="">
</body>
</html>"#,
        image_data_url
    );

    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/html; charset=utf-8")
        .header("Cache-Control", "public, max-age=31536000")
        .body(html_content.into())
        .map_err(|_| AppError::Internal("Failed to build response".to_string()))
}

pub async fn get_view_data(
    State((_, storage, image_repo, _)): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Response, AppError> {
    let image = image_repo
        .find_by_id(id)
        .await?
        .ok_or(AppError::ImageNotFound)?;

    let file_path = storage.get_file_path(&image.file_path);
    
    if !file_path.exists() {
        return Err(AppError::ImageNotFound);
    }

    let extension = file_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("application/octet-stream");
    
    let content_type = match extension {
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "gif" => "image/gif",
        "webp" => "image/webp",
        _ => "application/octet-stream",
    };

    let data = tokio::fs::read(&file_path)
        .await
        .map_err(|_| AppError::ImageNotFound)?;

    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", content_type)
        .header("Cache-Control", "public, max-age=31536000")
        .body(data.into())
        .map_err(|_| AppError::Internal("Failed to build response".to_string()))
}