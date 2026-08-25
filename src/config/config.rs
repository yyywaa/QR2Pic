use sqlx::migrate::Migrator;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::env;
use std::path::Path;

#[derive(Clone)]
pub struct Config {
    pub database_url: String,
    pub storage_path: String,
    pub storage_url: String,
    pub server_port: u16,
    pub delete_key: String,
}

impl Config {
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        dotenvy::dotenv().ok();

        let database_url = env::var("DATABASE_URL")?;
        let storage_path = env::var("STORAGE_PATH")
            .unwrap_or_else(|_| "/app/uploads".to_string());
        let storage_url = env::var("STORAGE_URL")
            .unwrap_or_else(|_| "http://localhost:3000/images".to_string());
        let server_port = env::var("SERVER_PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse::<u16>()?;
        let delete_key = env::var("DELETE_KEY")?;

        Ok(Config {
            database_url,
            storage_path,
            storage_url,
            server_port,
            delete_key,
        })
    }

    pub async fn create_db_pool(&self) -> Result<PgPool, Box<dyn std::error::Error>> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&self.database_url)
            .await?;
        
        match Migrator::new(Path::new("./migrations")).await {
            Ok(migrator) => {
                if let Err(e) = migrator.run(&pool).await {
                    tracing::warn!("Failed to run migrations: {}, continuing without migrations", e);
                }
            }
            Err(e) => {
                tracing::warn!("Failed to load migrations: {}, continuing without migrations", e);
            }
        }
        
        Ok(pool)
    }

    pub fn create_storage(&self) -> crate::storage::Storage {
        crate::storage::Storage::new(self.storage_path.clone(), self.storage_url.clone())
    }
}
