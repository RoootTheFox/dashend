mod api;
mod structs;

#[macro_use]
extern crate core;

#[macro_use]
extern crate rocket;

use rocket_db_pools::Database;

use crate::structs::GenericError;
use rocket::fairing::AdHoc;
use rocket::{Build, Config, Rocket};
use std::net::Ipv4Addr;

#[derive(Database)]
#[database("sqlx")]
struct Db(sqlx::SqlitePool);

async fn run_migrations(rocket: Rocket<Build>) -> rocket::fairing::Result {
    if let Some(db) = Db::fetch(&rocket) {
        match sqlx::migrate!().run(&db.0).await {
            Ok(_) => {}
            Err(e) => {
                eprintln!("failed to run migrations: {:?}", e);
                return Err(rocket);
            }
        }

        Ok(rocket)
    } else {
        Err(rocket)
    }
}

#[rocket::main]
async fn main() -> Result<(), GenericError> {
    let db_url = dotenvy::var("DATABASE_URL")?;
    let figment = Config::figment()
        .merge(("port", 61475))
        .merge(("address", Ipv4Addr::from([0, 0, 0, 0])))
        .merge((
            "databases.sqlx",
            rocket_db_pools::Config {
                url: db_url,
                min_connections: None,
                max_connections: 256,
                connect_timeout: 3,
                idle_timeout: None,
            },
        ));

    let _rocket = rocket::build()
        .attach(Db::init())
        .attach(AdHoc::try_on_ignite("DB Migrations", run_migrations))
        .mount("/", routes![api::profile::get_profile])
        .configure(figment)
        .launch()
        .await?;

    Ok(())
}
