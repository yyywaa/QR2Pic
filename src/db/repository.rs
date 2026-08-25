use super::models::{CreateImage, Image};
use crate::error::AppError;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct ImageRepository {
    pool: PgPool,
}

impl ImageRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, image: CreateImage) -> Result<Image, AppError> {
        let record = sqlx::query_as(
            "INSERT INTO images (file_path) VALUES ($1) RETURNING id, file_path, created_at"
        )
        .bind(image.file_path)
        .fetch_one(&self.pool)
        .await?;

        Ok(record)
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<Image>, AppError> {
        let image = sqlx::query_as(
            "SELECT id, file_path, created_at FROM images WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(image)
    }

    pub async fn delete_by_id(&self, id: Uuid) -> Result<bool, AppError> {
        let result = sqlx::query("DELETE FROM images WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }
}
