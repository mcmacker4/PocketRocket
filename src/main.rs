#[macro_use]
extern crate rocket;

use std::error::Error;
use std::fs;
use mysql::Pool;
use mysql::prelude::Queryable;
use serde::Deserialize;

#[get("/")]
fn hello() -> &'static str {
    "Hello, world!"
}

#[launch]
fn rocket() -> _ {
    let config = read_config().expect("Failed to read config");
    connect_db(&config).expect("Failed to connect to db");

    rocket::build()
        .mount("/", routes![hello])
}

#[derive(Debug, Deserialize)]
struct DatabaseConfig {
    host: String,
    port: u16,
    name: String,
    username: String,
    password: String,
}

#[derive(Debug, Deserialize)]
struct Config {
    database: DatabaseConfig,
}

fn read_config() -> Result<Config, Box<dyn Error>> {
    let file = fs::read_to_string("Rocket.toml")?;
    let config: Config = toml::from_str(&file)?;
    println!("{:?}", config);

    Ok(config)
}

fn connect_db(config: &Config) -> Result<(), Box<dyn Error>> {
    let db = &config.database;
    let url = format!("mysql://{}:{}@{}:{}/{}", db.username, db.password, db.host, db.port, db.name);
    let pool = Pool::new(url.as_str())?;

    let mut conn = pool.get_conn()?;

    // Let's create a table for payments.
    conn.query_drop(
        r"CREATE TABLE testing (
            id int not null,
            amount int not null,
            name text
        )")?;

    Ok(())
}