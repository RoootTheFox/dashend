use rocket::http::{ContentType, Status};
use rocket::response::Responder;
use rocket::{response, Request, Response};
use serde::{Deserialize, Serialize};
use std::io::Cursor;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GenericError {
    //#[error("invalid authentication")]
    //InvalidAuthenticationError,
    #[error("io error")]
    IOError(#[from] std::io::Error),

    #[error("missing environment variable")]
    MissingEnvVarError(#[from] std::env::VarError),

    #[error("database error")]
    GetMysqlErr(#[from] sqlx::Error),

    #[error("env error")]
    EnvError(#[from] dotenvy::Error),

    #[error("rocket error")]
    RocketError(#[from] rocket::Error),
}

#[derive(Serialize, Deserialize)]
pub struct ApiResponse<T> {
    success: bool,
    code: Status,
    message: String,
    data: Option<T>,
}

impl<T> From<T> for ApiResponse<T> {
    fn from(meow: T) -> ApiResponse<T> {
        ApiResponse {
            success: true,
            code: Status::Ok,
            message: "".to_string(),
            data: Some(meow),
        }
    }
}

impl GenericError {
    fn make_response(self, code: Status) -> response::Result<'static> {
        let message = self.to_string();
        self.make_response_msg(code, message)
    }
    fn make_response_msg(self, code: Status, message: String) -> response::Result<'static> {
        let err: ApiResponse<Option<String>> = ApiResponse {
            success: false,
            code,
            message,
            data: None,
        };
        let body = serde_json::to_string(&err)
            .unwrap_or(r#"{"success":false,"code":500,"message":"oops"}"#.to_string());

        Response::build()
            .header(ContentType::JSON)
            .status(code)
            .sized_body(body.len(), Cursor::new(body))
            .ok()
    }
}

impl<'r, 'o: 'r> Responder<'r, 'o> for GenericError {
    fn respond_to(self, req: &'r Request<'_>) -> response::Result<'o> {
        match self {
            //GenericError::InvalidAuthenticationError => self.make_response(Status::InternalServerError),
            GenericError::IOError(..) => self.make_response(Status::InternalServerError),
            GenericError::MissingEnvVarError(..) => self.make_response(Status::InternalServerError),
            GenericError::GetMysqlErr(ref e) => match e {
                sqlx::Error::RowNotFound => {
                    println!("sex");
                    self.make_response_msg(Status::NotFound, "not found".to_string())
                }
                _ => self.make_response(Status::InternalServerError),
            },

            _ => Status::InternalServerError.respond_to(req),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Profile {
    pub(crate) id: i64,
    pub(crate) bio: Option<String>,
    pub(crate) pronouns: Option<String>,
    // socials
    pub(crate) website: Option<String>,
    pub(crate) social_github: Option<String>,
    pub(crate) social_bluesky: Option<String>,
    pub(crate) social_fediverse: Option<String>,
    pub(crate) social_discord: Option<String>,
    pub(crate) social_matrix: Option<String>,
    pub(crate) social_tumblr: Option<String>,
    pub(crate) social_myspace: Option<String>,
    pub(crate) social_facebook: Option<String>,
}
