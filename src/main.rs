mod api;
mod structs;
mod utils;

#[macro_use]
extern crate core;

#[macro_use]
extern crate rocket;

use dashmap::DashMap;
use rocket_db_pools::Database;
use structs::Challenge;

use crate::structs::GenericError;
use rocket::fairing::AdHoc;
use rocket::{tokio, Build, Config, Rocket};
use std::collections::HashMap;
use std::net::Ipv4Addr;
use std::sync::Arc;

#[derive(Database)]
#[database("sqlx")]
struct Db(sqlx::SqlitePool);

struct AuthStuff {
    pending_challenges: Arc<DashMap<i64, Challenge>>,
    completed_challenges: Arc<DashMap<i64, String>>,
}

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

    let gd_account_id = dotenvy::var("GD_ACC_ID").unwrap();
    let mut hasher = sha1_smol::Sha1::new();
    // i love rob
    hasher.update(format!("{}mI29fmAnxgTs", dotenvy::var("GD_ACC_PW").unwrap()).as_bytes());
    let gd_account_gjp2 = hasher.digest().to_string();
    println!("gjp2: {}", gd_account_gjp2);

    let completed_challenges: Arc<DashMap<i64, String>> = DashMap::new().into();
    {
        let completed_challenges = Arc::clone(&completed_challenges); // clone so we can use it in the tokio closure
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(3));

            let client = reqwest::Client::builder()
                .default_headers({
                    let mut headers = reqwest::header::HeaderMap::new();
                    headers.insert("User-Agent", reqwest::header::HeaderValue::from_static(""));
                    headers
                })
                .build()
                .unwrap();
            let mut params = HashMap::new();
            params.insert("secret", "Wmfd2893gb7");
            params.insert("accountID", &gd_account_id);
            params.insert("gjp2", &gd_account_gjp2);

            loop {
                interval.tick().await;

                let response = client
                    .post("https://www.boomlings.com/database/getGJMessages20.php")
                    .form(&params)
                    .send()
                    .await
                    .unwrap();
                let response_code = response.status();
                let response_text = response.text().await.unwrap();
                if response_code != 200 || response_text == "-1" {
                    eprintln!("oopsie woopsie: {}: {}", response_code, response_text);
                }

                let messages = utils::parse_gj_messages_response(response_text);

                let mut message_deletion_string = "".to_string();
                messages.iter().for_each(|m| {
                    if m.subject.starts_with("auth-") {
                        let auth_code = m.subject.trim_start_matches("auth-");
                        println!("auth message from {} = {}", m.from, auth_code);
                        completed_challenges.insert(m.from, auth_code.to_string());
                    }

                    if message_deletion_string.len() > 0 {
                        message_deletion_string += ",";
                    }
                    message_deletion_string += &m.id;
                });

                if messages.len() > 0 {
                    // clean up!
                    let mut deletion_params = params.clone();
                    deletion_params.insert("messages", &message_deletion_string);

                    let response = client
                        .post("https://www.boomlings.com/database/deleteGJMessages20.php")
                        .form(&deletion_params)
                        .send()
                        .await
                        .unwrap();
                    let response_code = response.status();
                    let response_text = response.text().await.unwrap();
                    if response_code != 200 || response_text == "-1" {
                        eprintln!("oopsie woopsie: {}: {}", response_code, response_text);
                    }
                }
            }
            /*
            completed_challenges.iter().for_each(|a| {
                println!("{} -> {:?}", a.key(), a.value());
            });
            */
        });
    }

    let _rocket = rocket::build()
        .attach(Db::init())
        .attach(AdHoc::try_on_ignite("DB Migrations", run_migrations))
        .manage(AuthStuff {
            pending_challenges: DashMap::new().into(),
            completed_challenges,
        })
        .mount(
            "/",
            routes![
                api::profile::get_profile,
                api::auth::request_challenge,
                api::auth::challenge_complete
            ],
        )
        .configure(figment)
        .launch()
        .await?;

    Ok(())
}
