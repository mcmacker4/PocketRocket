#[macro_use]
extern crate rocket;

use figment::providers::{Format, Toml};
use mysql::prelude::Queryable;
use mysql::Pool;
use rocket::Config;
use serde::Deserialize;
use std::error::Error;

#[get("/")]
fn hello() -> &'static str {
    "Hello, world!"
}

#[launch]
fn rocket() -> _ {
    let figment = Config::figment().merge(Toml::file("Config.toml").nested());
    let db = figment
        .extract_inner::<DatabaseConfig>("database")
        .expect("Database config not found");

    connect_db(&db).expect("Could not connect to database");

    rocket::custom(figment).mount("/", routes![hello])
}

#[derive(Debug, Deserialize)]
struct DatabaseConfig {
    host: String,
    port: u16,
    name: String,
    username: String,
    password: String,
}

fn connect_db(db: &DatabaseConfig) -> Result<(), Box<dyn Error>> {
    let url = format!(
        "mysql://{}:{}@{}:{}/{}",
        db.username, db.password, db.host, db.port, db.name
    );

    println!("Connecting to database at: {}", url);

    let pool = Pool::new(url.as_str())?;

    let mut conn = pool.get_conn()?;

    // Let's create a table for payments.
    conn.query_drop(
        r"CREATE TABLE testing (
            id int not null,
            amount int not null,
            name text
        )",
    )?;

    Ok(())
}

