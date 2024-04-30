use rocket::http::Status;

#[catch(500)]
pub fn catch_500() -> String {
    Status::InternalServerError.to_string()
}

#[catch(400)]
pub fn catch_400() -> String {
    Status::BadRequest.to_string()
}

#[catch(401)]
pub fn catch_401() -> String {
    Status::Unauthorized.to_string()
}

#[catch(404)]
pub fn catch_404() -> String {
    Status::NotFound.to_string()
}

#[catch(422)]
pub fn catch_422() -> String {
    Status::UnprocessableEntity.to_string()
}