use std::sync::Arc;

use rocket::serde::Deserialize;
use sqlx::MySqlPool;

use crate::AnyError;

#[derive(Debug, Clone)]
pub struct Database {
    pool: MySqlPool,
    config: Arc<DatabaseConfig>,
}

#[derive(Debug, Deserialize)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub migrations_path: Option<String>,
    pub database: String,
}

impl Database {
    pub async fn connect(config: Arc<DatabaseConfig>) -> Result<Self, AnyError> {
        let pool = MySqlPool::connect(&Self::connection_url(&config)).await?;
        return Ok(Database { pool, config });
    }

    fn connection_url(config: &DatabaseConfig) -> String {
        format!(
            "mysql://{}:{}@{}:{}/{}",
            config.username, config.password, config.host, config.port, config.database
        )
    }
}

pub async fn run_migrations(db: &mut Database) -> Result<(), AnyError> {
    sqlx::migrate!("./schema").run(&db.pool).await?;
    Ok(())
}
