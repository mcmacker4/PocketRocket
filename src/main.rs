#[macro_use]
extern crate rocket;

use std::error::Error;
use std::sync::Arc;

use figment::providers::{Format, Toml};
use rocket::{Config, State, Rocket, Build};
use crate::api::ApiError;
use crate::database::{Database, DatabaseConfig};

mod database;
mod api;

pub type AnyError = Box<dyn Error>;

#[rocket::main]
async fn main() {
    let _ = rocket().launch().await;
}

/// Launches the Rocket application.
fn rocket() -> Rocket<Build> {
    let figment = Config::figment().merge(Toml::file("Config.toml").nested());
    let db_config = Arc::new(
        figment
            .extract_inner::<DatabaseConfig>("database")
            .expect("Database config not found")
    );

    let db = Database::connect(db_config).expect("Could not connect to database");

    rocket::custom(figment)
        .mount("/", routes![hello])
        .manage(db)
}

#[get("/")]
fn hello(db: &State<Database>) -> Result<String, ApiError> {
    let db_result: String = db.get_field("*SELECT 'Hello from the Database!'", ())?
        .unwrap_or("Database not working".to_string());

    Ok(db_result)
}