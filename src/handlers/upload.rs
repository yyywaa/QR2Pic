use axum::{
    extract::{Multipart, State},
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
);

const MAX_FILE_SIZE: u64 = 10 * 1024 * 1024;

pub async fn upload_image(
    State((_pool, storage, image_repo, _)): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<Value>, AppError> {
    let mut file_data = None;
    let mut file_name = None;

    while let Some(field) = multipart.next_field().await? {
        let field_name = field.name().unwrap_or("").to_string();
        
        if field_name == "file" {
            let name = field.file_name().map(|s: &str| s.to_string());
            let data: Vec<u8> = field.bytes().await?.to_vec();
            
            if data.len() as u64 > MAX_FILE_SIZE {
                return Err(AppError::FileTooLarge);
            }
            
            file_data = Some(data.to_vec());
            file_name = name;
            break;
        }
    }

    let file_data = file_data.ok_or(AppError::MissingFile)?;
    let file_name = file_name.ok_or(AppError::MissingFile)?;

    let extension = crate::storage::Storage::get_file_extension(&file_name)
        .ok_or(AppError::InvalidFileType)?;
    
    if !crate::storage::Storage::is_allowed_extension(&extension) {
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
