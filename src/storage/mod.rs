use crate::error::AppError;
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::io::AsyncWriteExt;

#[derive(Clone)]
pub struct Storage {
    base_path: PathBuf,
    base_url: String,
}

impl Storage {
    pub fn new(base_path: String, base_url: String) -> Self {
        Self {
            base_path: PathBuf::from(base_path),
            base_url,
        }
    }

    pub async fn init(&self) -> Result<(), AppError> {
        fs::create_dir_all(&self.base_path)
            .await
            .map_err(|e| AppError::Storage(format!("Failed to create storage directory: {}", e)))?;
        Ok(())
    }

    pub async fn save_file(
        &self,
        key: &str,
        content: Vec<u8>,
    ) -> Result<String, AppError> {
        let file_path = self.base_path.join(key);
        
        let mut file = fs::File::create(&file_path)
            .await
            .map_err(|e| AppError::Storage(format!("Failed to create file: {}", e)))?;
        
        file.write_all(&content)
            .await
            .map_err(|e| AppError::Storage(format!("Failed to write file: {}", e)))?;
        
        Ok(key.to_string())
    }

    pub async fn delete_file(&self, key: &str) -> Result<(), AppError> {
        let file_path = self.base_path.join(key);
        
        if file_path.exists() {
            fs::remove_file(&file_path)
                .await
                .map_err(|e| AppError::Storage(format!("Failed to delete file: {}", e)))?;
        }
        
        Ok(())
    }

    pub fn get_file_url(&self, key: &str) -> String {
        format!("{}/{}", self.base_url.trim_end_matches('/'), key)
    }

    pub fn get_file_path(&self, key: &str) -> PathBuf {
        self.base_path.join(key)
    }

    pub fn get_file_extension(filename: &str) -> Option<String> {
        Path::new(filename)
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_lowercase())
    }

    pub fn is_allowed_extension(extension: &str) -> bool {
        matches!(extension, "jpg" | "jpeg" | "png" | "gif" | "webp")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_file_extension_jpg() {
        assert_eq!(Storage::get_file_extension("photo.jpg"), Some("jpg".to_string()));
    }

    #[test]
    fn test_get_file_extension_jpeg() {
        assert_eq!(Storage::get_file_extension("image.JPEG"), Some("jpeg".to_string()));
    }

    #[test]
    fn test_get_file_extension_png() {
        assert_eq!(Storage::get_file_extension("/path/to/image.png"), Some("png".to_string()));
    }

    #[test]
    fn test_get_file_extension_with_uppercase() {
        assert_eq!(Storage::get_file_extension("Photo.JPG"), Some("jpg".to_string()));
    }

    #[test]
    fn test_get_file_extension_no_extension() {
        assert_eq!(Storage::get_file_extension("filename"), None);
    }

    #[test]
    fn test_get_file_extension_multiple_dots() {
        assert_eq!(Storage::get_file_extension("my.photo.jpg"), Some("jpg".to_string()));
    }

    #[test]
    fn test_get_file_extension_hidden_file() {
        assert_eq!(Storage::get_file_extension(".env"), None);
    }

    #[test]
    fn test_is_allowed_extension_jpg() {
        assert!(Storage::is_allowed_extension("jpg"));
    }

    #[test]
    fn test_is_allowed_extension_jpeg() {
        assert!(Storage::is_allowed_extension("jpeg"));
    }

    #[test]
    fn test_is_allowed_extension_png() {
        assert!(Storage::is_allowed_extension("png"));
    }

    #[test]
    fn test_is_allowed_extension_gif() {
        assert!(Storage::is_allowed_extension("gif"));
    }

    #[test]
    fn test_is_allowed_extension_webp() {
        assert!(Storage::is_allowed_extension("webp"));
    }

    #[test]
    fn test_is_allowed_extension_case_insensitive() {
        assert!(!Storage::is_allowed_extension("JPG"));
        assert!(!Storage::is_allowed_extension("PNG"));
    }

    #[test]
    fn test_is_not_allowed_extension_exe() {
        assert!(!Storage::is_allowed_extension("exe"));
    }

    #[test]
    fn test_is_not_allowed_extension_html() {
        assert!(!Storage::is_allowed_extension("html"));
    }

    #[test]
    fn test_is_not_allowed_extension_pdf() {
        assert!(!Storage::is_allowed_extension("pdf"));
    }

    #[test]
    fn test_is_not_allowed_extension_svg() {
        assert!(!Storage::is_allowed_extension("svg"));
    }

    #[test]
    fn test_is_not_allowed_extension_empty() {
        assert!(!Storage::is_allowed_extension(""));
    }
}
