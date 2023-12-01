use std::sync::Arc;
use mysql::{Params, Pool, Row};
use mysql::prelude::{FromRow, FromValue, Queryable};
use rocket::serde::Deserialize;
use crate::AnyError;

#[derive(Debug, Clone)]
pub struct Database {
    pool: Pool,
    config: Arc<DatabaseConfig>,
}

#[derive(Debug, Deserialize)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub name: String,
    pub username: String,
    pub password: String,
}

impl Database {
    pub fn connect(config: Arc<DatabaseConfig>) -> Result<Self, AnyError> {
        let url = format!(
            "mysql://{}:{}@{}:{}/{}",
            config.username, config.password, config.host, config.port, config.name
        );

        println!("Connecting to database at: {}", url);

        let pool = Pool::new(url.as_str())?;

        let mut conn = pool.get_conn()?;

        // Verify connection
        conn.query_drop(r"SELECT 1")?;

        Ok(Database { pool, config })
    }

    pub fn run(&self, query: &str) -> Result<(), AnyError> {
        let mut conn = self.pool.get_conn()?;
        conn.query_drop(query)?;
        Ok(())
    }

    pub fn get_row<T: FromRow, P: Into<Params>>(&self, query: &'static str, params: P) -> Result<Option<T>, AnyError> {
        let mut conn = self.pool.get_conn()?;
        let row = conn.exec_first(query, params)?;
        Ok(row)
    }

    pub fn get_field<T: FromValue, P: Into<Params>>(&self, query: &'static str, params: P) -> Result<Option<T>, AnyError> {
        let row: Option<Row> = self.get_row(query, params)?;
        if let Some(row) = row {
            let field: Option<T> = row.get(0);
            Ok(field)
        } else {
            Ok(None)
        }
    }}