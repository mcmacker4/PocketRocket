use std::collections::HashMap;
use std::hash::Hash;
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
    /// Attempt to connect to the database
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

    /// Execute a query that doesn't return any data
    pub fn run(&self, query: &str) -> Result<(), AnyError> {
        let mut conn = self.pool.get_conn()?;
        conn.query_drop(query)?;
        Ok(())
    }

    /// Execute a query that returns a single row, the rest are discarded
    pub fn get_row<T: FromRow, P: Into<Params>>(&self, query: &'static str, params: P) -> Result<Option<T>, AnyError> {
        let mut conn = self.pool.get_conn()?;
        let row = conn.exec_first(query, params)?;
        Ok(row)
    }

    /// Execute a query that returns a single row and takes the first field, the rest are discarded
    pub fn get_field<T: FromValue, P: Into<Params>>(&self, query: &'static str, params: P) -> Result<Option<T>, AnyError> {
        let row: Option<Row> = self.get_row(query, params)?;

        if let Some(row) = row {
            let field: Option<T> = row.get(0);
            Ok(field)
        } else {
            Ok(None)
        }
    }

    /// Execute a query that returns several rows and takes the first field of each row
    pub fn get_column<T: FromValue, P: Into<Params>>(&self, query: &'static str, params: P) -> Result<Vec<T>, AnyError> {
        let mut conn = self.pool.get_conn()?;
        let rows: Vec<Row> = conn.exec(query, params)?;
        let mut column: Vec<T> = Vec::new();

        for row in rows {
            let field: Option<T> = row.get(0);
            if let Some(field) = field {
                column.push(field);
            }
        }
        Ok(column)
    }

    /// Execute a query that returns several rows and takes the first two fields of each row and build a map
    pub fn get_map<K: FromValue, V: FromValue, P: Into<Params>>(&self, query: &'static str, params: P) -> Result<HashMap<K, V>, AnyError> where K: Eq + Hash {
        let mut conn = self.pool.get_conn()?;
        let rows: Vec<Row> = conn.exec(query, params)?;
        let mut map: HashMap<K, V> = HashMap::new();

        for row in rows {
            let key: Option<K> = row.get(0);
            let value: Option<V> = row.get(1);
            if let (Some(key), Some(value)) = (key, value) {
                map.insert(key, value);
            }
        }
        Ok(map)
    }

    /// Execute a query that returns several rows, each row is converted to a struct defined by the caller
    pub fn get_table<T: FromRow, P: Into<Params>>(&self, query: &'static str, params: P) -> Result<Vec<T>, AnyError> {
        let mut conn = self.pool.get_conn()?;
        let rows: Vec<Row> = conn.exec(query, params)?;
        let mut table: Vec<T> = Vec::new();

        for row in rows {
            let row: T = mysql::from_row(row);
            table.push(row);
        }
        Ok(table)
    }

    // Execute a query that returns several rows grouped in a map by a column
    pub fn get_grouped_by<K, V, P>(&self, query: &'static str, params: P, group_by_column: &'static str) -> Result<HashMap<K, Vec<V>>, AnyError>
        where K: Eq + Hash, K: FromValue, V: FromRow, P: Into<Params>
    {
        let mut conn = self.pool.get_conn()?;
        let rows: Vec<Row> = conn.exec(query, params)?;
        let mut map: HashMap<K, Vec<V>> = HashMap::new();

        for row in rows {
            let key: Option<K> = row.get(group_by_column);
            if let Some(key) = key {
                let value: V = mysql::from_row(row);
                let entry = map.entry(key).or_insert(Vec::new());
                entry.push(value);
            }
        }

        Ok(map)
    }
}
