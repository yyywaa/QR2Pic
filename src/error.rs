use axum::{
    extract::multipart::MultipartError as AxumMultipartError,
    http::StatusCode,
    response::{IntoResponse, Response},
};

#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Storage error: {0}")]
    Storage(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Multipart error: {0}")]
    Multipart(#[from] multer::Error),

    #[error("Axum multipart error: {0}")]
    AxumMultipart(#[from] AxumMultipartError),

    #[error("Invalid file type")]
    InvalidFileType,

    #[error("File too large")]
    FileTooLarge,

    #[error("Image not found")]
    ImageNotFound,

    #[error("Invalid delete key")]
    InvalidDeleteKey,

    #[error("Missing delete key header")]
    MissingDeleteKey,

    #[error("Missing file in upload")]
    MissingFile,

    #[error("Environment variable error: {0}")]
    EnvVar(#[from] std::env::VarError),

    #[error("Parse error: {0}")]
    Parse(#[from] std::num::ParseIntError),

    #[error("UUID error: {0}")]
    Uuid(#[from] uuid::Error),

    #[error("Internal server error: {0}")]
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match self {
            AppError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Storage(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Io(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Multipart(_) => StatusCode::BAD_REQUEST,
            AppError::AxumMultipart(_) => StatusCode::BAD_REQUEST,
            AppError::InvalidFileType => StatusCode::BAD_REQUEST,
            AppError::FileTooLarge => StatusCode::PAYLOAD_TOO_LARGE,
            AppError::ImageNotFound => StatusCode::NOT_FOUND,
            AppError::InvalidDeleteKey => StatusCode::UNAUTHORIZED,
            AppError::MissingDeleteKey => StatusCode::UNAUTHORIZED,
            AppError::MissingFile => StatusCode::BAD_REQUEST,
            AppError::EnvVar(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Parse(_) => StatusCode::BAD_REQUEST,
            AppError::Uuid(_) => StatusCode::BAD_REQUEST,
            AppError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        let body = format!("Error: {}", self);
        (status, body).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;

    fn get_status(error: AppError) -> StatusCode {
        error.into_response().status()
    }

    #[test]
    fn test_invalid_file_type_status() {
        assert_eq!(
            get_status(AppError::InvalidFileType),
            StatusCode::BAD_REQUEST
        );
    }

    #[test]
    fn test_file_too_large_status() {
        assert_eq!(
            get_status(AppError::FileTooLarge),
            StatusCode::PAYLOAD_TOO_LARGE
        );
    }

    #[test]
    fn test_image_not_found_status() {
        assert_eq!(get_status(AppError::ImageNotFound), StatusCode::NOT_FOUND);
    }

    #[test]
    fn test_invalid_delete_key_status() {
        assert_eq!(
            get_status(AppError::InvalidDeleteKey),
            StatusCode::UNAUTHORIZED
        );
    }

    #[test]
    fn test_missing_delete_key_status() {
        assert_eq!(
            get_status(AppError::MissingDeleteKey),
            StatusCode::UNAUTHORIZED
        );
    }

    #[test]
    fn test_missing_file_status() {
        assert_eq!(get_status(AppError::MissingFile), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_internal_error_status() {
        assert_eq!(
            get_status(AppError::Internal("test".to_string())),
            StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[test]
    fn test_error_message_format() {
        let error = AppError::InvalidFileType;
        let body = format!("{}", error);
        assert!(body.contains("Invalid file type"));
    }

    #[test]
    fn test_error_message_file_too_large() {
        let error = AppError::FileTooLarge;
        let body = format!("{}", error);
        assert!(body.contains("File too large"));
    }

    #[test]
    fn test_error_message_image_not_found() {
        let error = AppError::ImageNotFound;
        let body = format!("{}", error);
        assert!(body.contains("Image not found"));
    }
}
