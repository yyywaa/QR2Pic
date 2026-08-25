pub mod delete;
pub mod image;
pub mod restore;
pub mod upload;
pub mod view;

use axum::http::HeaderMap;
use crate::error::AppError;

/// 恒定时间字符串比较，避免密钥比较被时序侧信道探测。
fn constant_time_eq(a: &str, b: &str) -> bool {
    let (a, b) = (a.as_bytes(), b.as_bytes());
    if a.len() != b.len() {
        return false;
    }
    a.iter().zip(b.iter()).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

/// 校验请求头中的密钥。header 缺失或密钥不匹配都返回 401。
pub fn check_key(
    headers: &HeaderMap,
    header_name: &str,
    expected: &str,
) -> Result<(), AppError> {
    let provided = headers
        .get(header_name)
        .and_then(|v| v.to_str().ok())
        .ok_or(AppError::MissingDeleteKey)?;

    if !constant_time_eq(provided, expected) {
        return Err(AppError::InvalidDeleteKey);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constant_time_eq() {
        assert!(constant_time_eq("secret", "secret"));
        assert!(!constant_time_eq("secret", "Secret"));
        assert!(!constant_time_eq("secret", "secre"));
        assert!(!constant_time_eq("secret", "secret-longer"));
        assert!(!constant_time_eq("", "x"));
    }
}