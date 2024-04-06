use crate::structs::{ApiResponse, DBUser, GenericError, Profile};
use crate::Db;
use rocket::serde::json::Json;
use rocket_authorization::oauth::OAuth;
use rocket_authorization::{AuthError, Credential};
use rocket_db_pools::Connection;

#[get("/api/v1/profiles/<id>")]
pub async fn get_profile(
    mut conn: Connection<Db>,
    id: u32,
) -> Result<Json<ApiResponse<Profile>>, GenericError> {
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

    // token verified, update db

    sqlx::query_as!(
        Profile, "REPLACE INTO profiles (id, bio, pronouns, website, social_github, social_bluesky, social_fediverse, social_discord, social_matrix, social_tumblr, social_myspace, social_facebook) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        profile.id, profile.bio, profile.pronouns, profile.website, profile.social_github, profile.social_bluesky, profile.social_fediverse,
        profile.social_discord, profile.social_matrix, profile.social_tumblr, profile.social_myspace, profile.social_facebook
    )
    .execute(&mut **conn)
    .await?;    

    Ok(Json("".to_string().into()))
}
