use axum::{
    extract::DefaultBodyLimit,
    routing::{get, post, put, delete},
    Router,
};
use sqlx::PgPool;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

use crate::handlers;

/// 单个图片文件的最大字节数（10MB），upload/restore 共用。
pub const MAX_FILE_SIZE: usize = 10 * 1024 * 1024;

pub fn create_router(
    pool: PgPool,
    storage: crate::storage::Storage,
    delete_key: String,
    upload_key: Option<String>,
) -> Router {
    let image_repo = crate::db::ImageRepository::new(pool.clone());

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/health", get(health_check))
        .route("/upload", post(handlers::upload::upload_image))
        .route("/image/:id", get(handlers::image::get_image))
        .route("/view/:id", get(handlers::view::get_view))
        .route("/view-data/:id", get(handlers::view::get_view_data))
        .route("/delete/:id", delete(handlers::delete::delete_image))
        // restore 用 Bytes 提取器，受 DefaultBodyLimit 约束（默认仅 2MB），
        // 这里把上限提到 MAX_FILE_SIZE，在读取 body 前就拦截超大请求。
        .route(
            "/restore/:key",
            put(handlers::restore::restore_file)
                .route_layer(DefaultBodyLimit::max(MAX_FILE_SIZE)),
        )
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state((pool, storage, image_repo, delete_key, upload_key))
}

async fn health_check() -> &'static str {
    "OK"
}
