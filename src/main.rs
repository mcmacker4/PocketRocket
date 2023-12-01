// Remove warnings of unused code, they will be used in the future, no need to remove or comment them out.
#![allow(dead_code)]
// Don't allow results without unwrap or proper error handling
#![deny(unused_must_use)]

#[macro_use]
extern crate rocket;

use std::error::Error;
use std::sync::Arc;

use crate::api::ApiError;
use crate::database::{Database, DatabaseConfig};
use figment::providers::{Format, Toml};
use rocket::{Build, Config, Rocket, State};

mod api;
mod database;

pub type AnyError = Box<dyn Error>;

#[rocket::main]
async fn main() {
    let _ = rocket().await.launch().await;
}

/// Launches the Rocket application.
async fn rocket() -> Rocket<Build> {
    let figment = Config::figment().merge(Toml::file("Config.toml").nested());
    let db_config = Arc::new(
        figment
            .extract_inner::<DatabaseConfig>("database")
            .expect("Database config not found"),
    );

    let mut db = Database::connect(db_config.clone())
        .await
        .expect("Could not connect to the database");

    database::run_migrations(&mut db)
        .await
        .expect("Could not execute migrations");

    rocket::custom(figment)
        .mount("/", routes![hello])
        .manage(db)
}

#[get("/")]
fn hello(db: &State<Database>) -> Result<String, ApiError> {
    Ok("Hello World".to_string())
}
