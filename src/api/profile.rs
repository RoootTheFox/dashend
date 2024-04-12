use std::time::{SystemTime, UNIX_EPOCH};

use crate::structs::{ApiResponse, DBUser, DBUserMisc, GenericError, Profile};
use crate::utils::check_discord_username;
use crate::Db;
use lazy_static::lazy_static;
use regex::Regex;
use rocket::serde::json::Json;
use rocket_authorization::oauth::OAuth;
use rocket_authorization::{AuthError, Credential};
use rocket_db_pools::Connection;
lazy_static! {
    static ref PRONOUNS_REGEX: Regex = Regex::new(
        r"^(s?he|the[ym]|it)/(her|s?he|him|the[ym]|its?)(/(her|s?he|him|the[ym]|its?))?$"
    )
    .expect("nuh uh?");
}

#[get("/api/v1/profiles/<id>")]
pub async fn get_profile(
    mut conn: Connection<Db>,
    id: u32,
) -> Result<Json<ApiResponse<Profile>>, GenericError> {
    let time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("")
        .as_secs();
    let time = time - 86400;

    let user_misc = sqlx::query_as!(DBUserMisc, "SELECT * FROM user_misc WHERE id = ?", id)
        .fetch_one(&mut **conn)
        .await?;
    let profile = sqlx::query_as!(Profile, "SELECT * FROM profiles WHERE id = ?", id)
        .fetch_one(&mut **conn)
        .await?;

    if time < user_misc.check_timeout.expect("") as u64 {
        println!("not checking (less than 24 hours have passed)")
    } else {
        println!("checking");
        check_discord_username(
            &mut conn,
            profile.social_discord.unwrap_or("".to_string()),
            id,
        )
        .await;
    }

    let profile = sqlx::query_as!(Profile, "SELECT * FROM profiles WHERE id = ?", id)
        .fetch_one(&mut **conn)
        .await?;

    Ok(Json(profile.into()))
}

#[post(
    "/api/v1/profiles/<id>",
    format = "application/json",
    data = "<profile>"
)]
pub async fn set_profile(
    mut conn: Connection<Db>,
    auth: Result<Credential<OAuth>, AuthError>,
    id: u32,
    profile: Json<Profile>,
) -> Result<Json<ApiResponse<String>>, GenericError> {
    let auth = auth?;

    // verify token - todo: move this into a seperate function
    sqlx::query_as!(
        DBUser,
        "SELECT * FROM users WHERE token = ? AND id = ?",
        auth.token,
        id
    )
    .fetch_one(&mut **conn)
    .await
    .map_err(|e| {
        println!("err {}", e);
        GenericError::InvalidAuthenticationError
    })?;

    let profile = profile.into_inner();
    if profile.id != id as i64 {
        println!("id does NOT match !!");
        return Err(GenericError::InvalidAuthenticationError);
    }

    // token verified, check data
    let pronouns = profile.pronouns.clone().unwrap_or("".to_string());
    if pronouns != "" && !PRONOUNS_REGEX.is_match(&pronouns) {
        eprintln!(
            "requested update with invalid pronouns: {} (does not match regex)",
            pronouns
        );
        return Err(GenericError::InvalidPronounsError);
    }

    sqlx::query_as!(
        Profile, "REPLACE INTO profiles (id, bio, pronouns, website, social_github, social_bluesky, social_fediverse, social_discord, social_matrix, social_tumblr, social_myspace, social_facebook) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        profile.id, profile.bio, profile.pronouns, profile.website, profile.social_github, profile.social_bluesky, profile.social_fediverse,
        profile.social_discord, profile.social_matrix, profile.social_tumblr, profile.social_myspace, profile.social_facebook
    )
    .execute(&mut **conn)
    .await?;

    Ok(Json("".to_string().into()))
}
