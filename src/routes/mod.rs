use axum::{
    routing::{get, post, put, delete},
    Router,
};
use sqlx::PgPool;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

use crate::handlers;

pub fn create_router(
    pool: PgPool, 
    storage: crate::storage::Storage, 
    delete_key: String
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
        .route("/restore/:key", put(handlers::restore::restore_file))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state((pool, storage, image_repo, delete_key))
}

async fn health_check() -> &'static str {
    "OK"
}
