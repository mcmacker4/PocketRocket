#[macro_use]
extern crate rocket;

use std::error::Error;

use figment::providers::{Format, Toml};
use mysql::prelude::Queryable;
use mysql::Pool;
use rocket::{http::Status, Config, State};
use serde::Deserialize;

#[get("/")]
fn hello(db: &State<Database>) -> Result<String, (Status, &'static str)> {
    if let Ok(mut conn) = db.pool.get_conn() {
        if let Ok(Some(msg)) = conn.query_first("SELECT 'Hello from the Database!'") {
            Ok(msg)
        } else {
            Err((Status::InternalServerError, "Query failed"))
        }
    } else {
        Err((
            Status::InternalServerError,
            "Could not get connection to the database",
        ))
    }
}

#[launch]
fn rocket() -> _ {
    let figment = Config::figment().merge(Toml::file("Config.toml").nested());
    let db_config = figment
        .extract_inner::<DatabaseConfig>("database")
        .expect("Database config not found");

    let db = connect_db(&db_config).expect("Could not connect to database");

    rocket::custom(figment)
        .mount("/", routes![hello])
        .manage(db)
}

#[derive(Debug, Deserialize)]
struct DatabaseConfig {
    host: String,
    port: u16,
    name: String,
    username: String,
    password: String,
}

#[derive(Debug, Clone)]
struct Database {
    pool: Pool,
}

fn connect_db(db: &DatabaseConfig) -> Result<Database, Box<dyn Error>> {
    let url = format!(
        "mysql://{}:{}@{}:{}/{}",
        db.username, db.password, db.host, db.port, db.name
    );

    println!("Connecting to database at: {}", url);

    let pool = Pool::new(url.as_str())?;

    let mut conn = pool.get_conn()?;

    // Verify connection
    conn.query_drop(r"SELECT 1")?;

    Ok(Database { pool })
}
