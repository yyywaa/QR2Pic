use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use crate::error::AppError;
use uuid::Uuid;

type AppState = (
    sqlx::PgPool,
    crate::storage::Storage,
    crate::db::ImageRepository,
    String,
    Option<String>,
);

pub async fn delete_image(
    State((_, storage, image_repo, delete_key, _)): State<AppState>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
) -> Result<Response, AppError> {
    crate::handlers::check_key(&headers, "X-Delete-Key", &delete_key)?;

    let image = image_repo
        .find_by_id(id)
        .await?
        .ok_or(AppError::ImageNotFound)?;

    storage.delete_file(&image.file_path).await?;

    let deleted = image_repo.delete_by_id(id).await?;
    
    if deleted {
        Ok((StatusCode::NO_CONTENT, ()).into_response())
    } else {
        Err(AppError::ImageNotFound)
    }
}
