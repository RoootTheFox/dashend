use crate::structs::{ApiResponse, Challenge, GenericError};
use crate::{AuthStuff, Db};
use rand::distributions::{Alphanumeric, DistString};
use rocket::serde::json::Json;
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
        None => return Err(GenericError::InvalidAuthenticationError),
    };

    let acc_id = challenge.key();
    println!("account id: {}", acc_id);

    let completed_challenge = match auth_stuff_meow.completed_challenges.get(acc_id) {
        Some(c) => c,
        None => {
            println!(
                "did NOT find challenge for {} in completed challenges",
                acc_id
            );
            return Err(GenericError::InvalidAuthenticationError);
        }
    };

    if completed_challenge.value() == &challenge.challenge {
        println!("woohoo !!");
    } else {
        println!("nuh uh");
    }

    println!("challenge key {:?}", challenge.key());

    Ok(Json("meow".to_string().into()))
}
