use crate::structs::{ApiResponse, Challenge, GenericError};
use crate::{AuthStuff, Db};
use rand::distributions::{Alphanumeric, DistString};
use rocket::serde::json::Json;
use rocket::tokio::time::{sleep, Duration};
use rocket::State;
use rocket_db_pools::Connection;
use uuid::Uuid;

#[get("/api/v1/request_challenge/<gd_acc>")]
pub async fn request_challenge(
    authmap_meow: &State<AuthStuff>,
    gd_acc: i64,
) -> Result<Json<ApiResponse<Challenge>>, GenericError> {
    let bot_id = dotenvy::var("GD_ACC_ID").unwrap().parse().unwrap();
    let meow = Challenge {
        bot_account_id: bot_id,
        challenge: Alphanumeric.sample_string(&mut rand::thread_rng(), 16),
        id: Uuid::new_v4(),
    };

    // todo: make auth challenges expire after ~30 seconds
    /*if authmap_meow.pending_challenges.contains_key(&gd_acc) {
        return Err(GenericError::AuthChallengeAlreadyRequested);
    }*/

    authmap_meow.pending_challenges.insert(gd_acc, meow.clone());
    Ok(Json(meow.into()))
}

#[get("/api/v1/challenge_complete/<challenge_id>")]
pub async fn challenge_complete(
    mut conn: Connection<Db>,
    auth_stuff_meow: &State<AuthStuff>,
    challenge_id: &str,
) -> Result<Json<ApiResponse<String>>, GenericError> {
    let uuid: Uuid = challenge_id.parse()?;
    let challenge = match auth_stuff_meow
        .pending_challenges
        .iter()
        .find(|nya| nya.id == uuid)
    {
        Some(a) => a,
        None => {
            println!("didn't find challenge");
            return Err(GenericError::InvalidAuthenticationError);
        },
    };

    let acc_id = challenge.key();

    let mut tries = 0;
    loop {
        if tries >= 7 {
            println!("oopsies (ran out of tries)");
            return Err(GenericError::InvalidAuthenticationError);
        }
        tries += 1;

        let completed_challenge = match auth_stuff_meow.completed_challenges.get(acc_id) {
            Some(c) => (c.key().clone(), c.value().clone()), // CLONE HERE since we do NOT want to have a reference (read below)
            None => {
                sleep(Duration::from_millis(500)).await;
                continue;
            }
        };

        if completed_challenge.1 == challenge.challenge {
            // !! this will block if a reference to completed_challenges is still alive !!
            auth_stuff_meow.completed_challenges.remove(acc_id);

            // todo: token stuff
            let token = Alphanumeric.sample_string(&mut rand::thread_rng(), 32);
            sqlx::query!(
                "REPLACE INTO users (id, token) VALUES (?, ?)",
                completed_challenge.0,
                token,
            )
            .execute(&mut **conn)
            .await?;
            return Ok(Json(token.into()));
        } else {
            println!("nuh uh ???? {} =/= {}", completed_challenge.1, challenge.challenge);
            return Err(GenericError::InvalidAuthenticationError);
        }
    }
}
