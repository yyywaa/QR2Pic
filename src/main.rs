use qr2pic::config::Config;
use qr2pic::routes::create_router;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let config = Config::from_env()?;
    tracing::info!("Starting QR2Pic server on port {}", config.server_port);

    let pool = config.create_db_pool().await?;
    let storage = config.create_storage();
    storage.init().await?;

    let app = create_router(
        pool, 
        storage, 
        config.delete_key
    );

    let addr = SocketAddr::from(([0, 0, 0, 0], config.server_port));
    tracing::info!("Server listening on {}", addr);

    let listener = TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
