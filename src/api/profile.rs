use crate::structs::{ApiResponse, GenericError, Profile};
use crate::Db;
use rocket::serde::json::Json;
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
