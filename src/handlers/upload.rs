use axum::{
    extract::{Multipart, State},
    http::HeaderMap,
    response::Json,
};
use crate::error::AppError;
use serde_json::{json, Value};
use uuid::Uuid;

type AppState = (
    sqlx::PgPool,
    crate::storage::Storage,
    crate::db::ImageRepository,
    String,
    Option<String>,
);

pub async fn upload_image(
    State((_pool, storage, image_repo, _, upload_key)): State<AppState>,
    headers: HeaderMap,
    mut multipart: Multipart,
) -> Result<Json<Value>, AppError> {
    // 设置了 UPLOAD_KEY 时，/upload 必须携带匹配的 X-Upload-Key 头
    if let Some(expected) = &upload_key {
        crate::handlers::check_key(&headers, "X-Upload-Key", expected)?;
    }

    let mut file_data = None;
    let mut file_name = None;

    while let Some(field) = multipart.next_field().await? {
        let field_name = field.name().unwrap_or("").to_string();

        if field_name == "file" {
            let name = field.file_name().map(|s: &str| s.to_string());
            // 流式读取：边读边累计字节数，超限立即返回，
            // 超大 body 不会先被整个读进内存
            let mut field = field;
            let mut data = Vec::new();
            while let Some(chunk) = field.chunk().await? {
                if data.len() + chunk.len() > crate::routes::MAX_FILE_SIZE {
                    return Err(AppError::FileTooLarge);
                }
                data.extend_from_slice(&chunk);
            }
            file_data = Some(data);
            file_name = name;
            break;
        }
    }

    let file_data = file_data.filter(|d: &Vec<u8>| !d.is_empty()).ok_or(AppError::MissingFile)?;
    let file_name = file_name.ok_or(AppError::MissingFile)?;

    let extension = crate::storage::Storage::get_file_extension(&file_name)
        .ok_or(AppError::InvalidFileType)?;

    if !crate::storage::Storage::is_allowed_extension(&extension) {
        return Err(AppError::InvalidFileType);
    }

    // 校验文件内容魔数，防止任意文件改扩展名伪装成图片（jpeg 归一为 jpg 比较）
    let ext_normalized = if extension == "jpeg" { "jpg" } else { extension.as_str() };
    if crate::storage::Storage::sniff_image_type(&file_data) != Some(ext_normalized) {
        return Err(AppError::InvalidFileType);
    }

    let id = Uuid::new_v4();
    let key = format!("{}.{}", id, extension);

    storage.save_file(&key, file_data).await?;

    let image_url = storage.get_file_url(&key);

    let image = image_repo
        .create(crate::db::CreateImage { file_path: key.clone() })
        .await?;

    Ok(Json(json!({
        "id": image.id,
        "url": image_url
    })))
}
