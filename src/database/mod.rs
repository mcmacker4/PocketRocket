use rocket::{
    fairing::{Fairing, Info, Kind},
    Orbit, Rocket,
};
use rocket_db_pools::Database;
use sqlx::migrate;

#[derive(Database)]
#[database("pocket_rocket")]
pub struct PocketDB(sqlx::MySqlPool);

pub struct PocketDBMigrationsFairing;

#[rocket::async_trait]
impl Fairing for PocketDBMigrationsFairing {
    fn info(&self) -> Info {
        Info {
            name: "Database Migrations Fairing",
            kind: Kind::Liftoff,
        }
    }

    async fn on_liftoff(&self, rocket: &Rocket<Orbit>) {
        println!("Running migrations fairing");
        if let Some(db) = PocketDB::fetch(&rocket) {
            println!("Running migrations");
            migrate!("./schema")
                .run(&**db)
                .await
                .expect("Could not run migrations");
        } else {
            println!("Could not find database state in rocket")
        }
    }
}
