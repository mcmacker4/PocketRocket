// Remove warnings of unused code, they will be used in the future, no need to remove or comment them out.
#![allow(dead_code)]
// Don't allow results without unwrap or proper error handling
#![deny(unused_must_use)]

#[macro_use]
extern crate rocket;

use std::error::Error;

use database::{PocketDB, PocketDBMigrationsFairing};
use figment::providers::{Format, Toml};
use rocket::{Build, Config, Rocket};
use rocket_db_pools::Database;

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

    rocket::custom(figment)
        .attach(PocketDB::init())
        .attach(PocketDBMigrationsFairing)
        .mount("/", routes![api::hello])
}
