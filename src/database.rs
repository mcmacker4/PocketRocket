use std::collections::HashMap;
use std::fs;
use std::hash::Hash;
use std::path::PathBuf;
use std::sync::Arc;
use mysql::{Params, Pool, Row};
use mysql::prelude::{FromRow, FromValue, Queryable};
use rocket::serde::Deserialize;
use crate::AnyError;

const DATABASE_NAME: &'static str = "pocket_rocket";

#[derive(Debug, Clone)]
pub struct Database {
    pool: Pool,
    config: Arc<DatabaseConfig>,
}

#[derive(Debug, Deserialize)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub migrations_path: Option<String>,
}

impl Database {
    /// Attempt to connect to the database
    pub fn connect(config: Arc<DatabaseConfig>) -> Result<Self, AnyError> {
        {
            let url = format!(
                "mysql://{}:{}@{}:{}",
                config.username, config.password, config.host, config.port
            );

            info!("Connecting to database at: {}", url);

            let pool = Pool::new(url.as_str())?;
            let mut conn = pool.get_conn()?;

            // Create default database if it doesn't exist
            conn.query_drop(r#"
                create database if not exists pocket_rocket
                    default character set utf8mb4
                    collate utf8mb4_unicode_ci"#
            )?;

            conn.query_drop(r#"
                create table if not exists pocket_rocket.migrations
                (
                    id         int primary key auto_increment comment "Unique migration ID",
                    filename   varchar(255) not null comment "File name of the migration",
                    version    int unique   not null comment "Version of the migration",
                    created_at timestamp default current_timestamp comment "Date and time when this row was inserted"
                )"#
            )?;
        }

        let url = format!(
            "mysql://{}:{}@{}:{}/{}",
            config.username, config.password, config.host, config.port, DATABASE_NAME
        );

        let pool = Pool::new(url.as_str())?;

        Ok(Database { pool, config })
    }

    /// Execute all the migrations that haven't been executed yet
    pub fn run_migrations(&mut self) -> Result<(), AnyError> {
        let migrations_path_config = self.config.migrations_path.clone().unwrap_or_else(|| "./schema".to_string());
        let migrations_path = PathBuf::from(migrations_path_config);

        info!("Running migrations from: {:?}", migrations_path);

        if !self.table_exists("migrations")? {
            let mut init = migrations_path.clone();
            init.push("00_init.sql");

            self.run_script(&init)?;
        }

        let mut migrations: Vec<(String, PathBuf, u32)> = vec![];

        for dir_entry in fs::read_dir(migrations_path)? {
            let dir_entry = dir_entry?;
            let filename = dir_entry.file_name().to_string_lossy().to_string();

            if !filename.contains("_") || !filename.ends_with(".sql") {
                continue;
            }

            let full_path = dir_entry.path();
            let version = filename.split('_').next().unwrap()
                .parse::<u32>().expect("File name must start with a number");

            migrations.push((filename, full_path, version));
        }

        migrations.sort_by(|a, b| a.2.cmp(&b.2));

        let mut last_migration: u32 = self.get_field(
            "select version from migrations order by version desc limit 1", ()
        )?.unwrap_or(0);

        for (filename, full_path, version) in migrations {
            if version <= last_migration { continue; }

            self.run_script(&full_path)?;
            self.run("insert into migrations (version, filename) values (?, ?)", (version, filename))?;
            last_migration = version;
        }

        Ok(())
    }

    // Check if a table exists
    pub fn table_exists(&mut self, table_name: &'static str) -> Result<bool, AnyError> {
        let result: Option<u32> = self.get_field(
            "select 1 from information_schema.tables where table_schema = ? and table_name = ?",
            (DATABASE_NAME, table_name)
        )?;

        Ok(result.is_some())
    }

    // Check if a table column exists
    pub fn table_column_exists(&mut self, table_name: &'static str, column: &'static str) -> Result<bool, AnyError> {
        let result: Option<u32> = self.get_field(
            "select 1 from information_schema.columns where table_schema = ? and table_name = ? and column_name = ?",
            (DATABASE_NAME, table_name, column)
        )?;

        Ok(result.is_some())
    }

    // Execute a script
    pub fn run_script(&mut self, script_path: &PathBuf) -> Result<(), AnyError> {
        let mut conn = self.pool.get_conn()?;
        let script = fs::read_to_string(script_path)?;
        conn.exec_drop(&script, ())?;
        Ok(())
    }

    /// Execute a query that doesn't return any data
    pub fn run<P: Into<Params>>(&self, query: &'static str, params: P) -> Result<(), AnyError> {
        let mut conn = self.pool.get_conn()?;
        conn.exec_drop(query, params)?;
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
