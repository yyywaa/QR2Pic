use axum::{
    body::Bytes,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use crate::error::AppError;

type AppState = (
    sqlx::PgPool,
    crate::storage::Storage,
    crate::db::ImageRepository,
    String,
    Option<String>,
);

/// 按指定文件名恢复图片文件到存储（仅写文件，不动数据库）。
/// 用于灾难恢复：数据库记录还在、文件丢失时，按 images.file_path 把原图放回去。
/// 鉴权复用 X-Delete-Key。body 大小由路由层的 DefaultBodyLimit 限制。
pub async fn restore_file(
    State((_, storage, _, delete_key, _)): State<AppState>,
    Path(key): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Response, AppError> {
    crate::handlers::check_key(&headers, "X-Delete-Key", &delete_key)?;

    // 防止路径穿越，key 必须是单纯的 "<uuid>.<ext>" 文件名
    if key.contains('/') || key.contains('\\') || key.contains("..") {
        return Err(AppError::InvalidFileType);
    }
    let extension = crate::storage::Storage::get_file_extension(&key)
        .ok_or(AppError::InvalidFileType)?;
    if !crate::storage::Storage::is_allowed_extension(&extension) {
        return Err(AppError::InvalidFileType);
    }

    if body.is_empty() {
        return Err(AppError::MissingFile);
    }
    if body.len() > crate::routes::MAX_FILE_SIZE {
        return Err(AppError::FileTooLarge);
    }

    storage.save_file(&key, body.to_vec()).await?;

    Ok((
        StatusCode::CREATED,
        axum::Json(serde_json::json!({ "key": key })),
    )
        .into_response())
}
