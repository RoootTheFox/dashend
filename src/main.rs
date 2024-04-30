mod api;
mod structs;
mod utils;
mod catchers;

#[macro_use]
extern crate core;

#[macro_use]
extern crate rocket;

use rocket::http::Header;
use rocket_db_pools::Database;
use structs::Challenge;
use timedmap::TimedMap;

use crate::structs::GenericError;
use rocket::fairing::{AdHoc, Fairing, Info, Kind};
use rocket::{tokio, Build, Config, Request, Response, Rocket};
use std::collections::HashMap;
use std::net::Ipv4Addr;
use std::process::exit;
use std::sync::Arc;
use std::time::Duration;

lazy_static::lazy_static! {
    static ref BOOMLINGS_SERVER: String = dotenvy::var("BOOMLINGS_SERVER_OVERRIDE").unwrap_or("https://www.boomlings.com".to_string());
}

#[derive(Database)]
#[database("sqlx")]
struct Db(sqlx::MySqlPool);

struct AuthStuff {
    pending_challenges: Arc<TimedMap<i64, Challenge>>,
    completed_challenges: Arc<TimedMap<i64, String>>,
}

pub struct CORS;

#[rocket::async_trait]
impl Fairing for CORS {
    fn info(&self) -> Info {
        Info {
            name: "i hate CORS",
            kind: Kind::Response,
        }
    }

    async fn on_response<'r>(&self, _request: &'r Request<'_>, response: &mut Response<'r>) {
        response.set_header(Header::new("Access-Control-Allow-Origin", "*"));
        response.set_header(Header::new(
            "Access-Control-Allow-Methods",
            "POST, GET, PATCH, OPTIONS",
        ));
        response.set_header(Header::new("Access-Control-Allow-Headers", "*"));
        response.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
    }
}

#[options("/<_..>")]
fn all_options() {
    /* Intentionally left empty */
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

    let gd_account_id = dotenvy::var("GD_ACC_ID")?;
    let mut hasher = sha1_smol::Sha1::new();
    // i love rob
    hasher.update(format!("{}mI29fmAnxgTs", dotenvy::var("GD_ACC_PW")?).as_bytes());
    let gd_account_gjp2 = hasher.digest().to_string();
    println!("gjp2: {}", gd_account_gjp2);

    let completed_challenges: Arc<TimedMap<i64, String>> = TimedMap::new().into();
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
                .unwrap_or_else(|e| {
                    eprintln!("failed to create reqwest client wtf ({:?}) !", e);
                    exit(1)
                });
            let mut params = HashMap::new();
            params.insert("secret", "Wmfd2893gb7");
            params.insert("accountID", &gd_account_id);
            params.insert("gjp2", &gd_account_gjp2);

            let mut message_deletion_string = "".to_string();
            loop {
                interval.tick().await;

                // clean up **BEFORE** we get messages
                if !message_deletion_string.is_empty() {
                    let mut deletion_params = params.clone();
                    deletion_params.insert("messages", &message_deletion_string);

                    let response = match client
                        .post(format!(
                            "{}/database/deleteGJMessages20.php",
                            utils::proxy_list().await.unwrap()
                        ))
                        .form(&deletion_params)
                        .send()
                        .await
                    {
                        Ok(r) => r,
                        Err(e) => {
                            eprintln!("failed to delete messages: {}", e);
                            continue;
                        }
                    };
                    let response_code = response.status();
                    let response_text = response.text().await.unwrap();
                    if response_code != 200 || response_text == "-1" {
                        eprintln!("oopsie woopsie: {}: {}", response_code, response_text);
                    }
                }

                let response = match client
                    .post(format!(
                        "{}/database/getGJMessages20.php",
                        utils::proxy_list().await.unwrap()
                    ))
                    .form(&params)
                    .send()
                    .await
                {
                    Ok(r) => r,
                    Err(e) => {
                        eprintln!("failed to request messages: {}", e);
                        continue;
                    }
                };
                let response_code = response.status();
                let response_text = response.text().await.unwrap();
                if response_code != 200 || response_text == "-1" {
                    eprintln!("oopsie woopsie: {}: {}", response_code, response_text);
                }

                let messages = match utils::parse_gj_messages_response(response_text) {
                    Ok(msgs) => msgs,
                    Err(e) => {
                        eprintln!("failed getting messages!! {:?}", e);
                        continue;
                    }
                };

                message_deletion_string = "".to_string();
                messages.iter().for_each(|m| {
                    if m.subject.starts_with("auth-") {
                        let auth_code = m.subject.trim_start_matches("auth-");
                        println!("auth message from {} = {}", m.from, auth_code);

                        while completed_challenges.contains(&m.from) {
                            completed_challenges.remove(&m.from);
                        }
                        completed_challenges.insert(
                            m.from,
                            auth_code.to_string(),
                            Duration::from_secs(8),
                        );
                    }

                    if !message_deletion_string.is_empty() {
                        message_deletion_string += ",";
                    }
                    message_deletion_string += &m.id;
                });
            }
        });
    }

    let _rocket = rocket::build()
        .attach(Db::init())
        .attach(AdHoc::try_on_ignite("DB Migrations", run_migrations))
        .attach(CORS)
        .manage(AuthStuff {
            pending_challenges: TimedMap::new().into(),
            completed_challenges,
        })
        .register("/", catchers!(catchers::catch_500, catchers::catch_401, catchers::catch_400, catchers::catch_404, catchers::catch_422))
        .mount(
            "/",
            routes![
                api::profile::get_profile,
                api::profile::set_profile,
                api::auth::request_challenge,
                api::auth::challenge_complete,
                all_options // fuck you cors
            ],
        )
        .configure(figment)
        .launch()
        .await?;

    Ok(())
}
