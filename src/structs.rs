use rocket::http::{ContentType, Status};
use rocket::response::Responder;
use rocket::{response, Request, Response};
use serde::{Deserialize, Serialize};
use std::io::Cursor;
use std::num::ParseIntError;
use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum GenericError {
    /*#[error("challenge already requested, try again later")]
    AuthChallengeAlreadyRequested,*/
    #[error("invalid authentication")]
    InvalidAuthenticationError,

    #[error("missing auth header")]
    MissingAuthHeaderError(#[from] rocket_authorization::AuthError),

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

    #[error("invalid uuid")]
    UuidError(#[from] uuid::Error),

    #[error("invalid digit")]
    ParseIntError(#[from] ParseIntError),

    /*#[error("invalid pronouns")]
    InvalidPronounsError,*/
    #[error("profanity")]
    ProfanityError(ProfanityErrorType),
}

#[derive(Debug, Copy, Clone)]
pub enum ProfanityErrorType {
    Bio,
    Pronouns,
}

#[derive(Serialize, Deserialize)]
pub struct ApiResponse<T> {
    success: bool,
    message: String,
    data: Option<T>,
}

impl<T> From<T> for ApiResponse<T> {
    fn from(meow: T) -> ApiResponse<T> {
        ApiResponse {
            success: true,
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
            GenericError::InvalidAuthenticationError => {
                self.make_response(Status::InternalServerError)
            }
            GenericError::UuidError(..) => {
                self.make_response_msg(Status::BadRequest, "invalid uuid".to_string())
            }
            GenericError::IOError(..) => self.make_response(Status::InternalServerError),
            GenericError::MissingEnvVarError(..) => self.make_response(Status::InternalServerError),
            GenericError::GetMysqlErr(ref e) => match e {
                sqlx::Error::RowNotFound => {
                    println!("sex");
                    self.make_response_msg(Status::NotFound, "not found".to_string())
                }
                _ => self.make_response(Status::InternalServerError),
            },
            GenericError::ProfanityError(where_) => self.make_response_msg(
                Status::BadRequest,
                format!(
                    "The {} you tried to set contains <cr>profanity</c>.\n \
                Note that <cy>trying to avoid the filter</c> will result in a <cr>ban</c>.\n \
                If you think this is a false-positive, please contact rooot.",
                    match where_ {
                        ProfanityErrorType::Bio => "bio",
                        ProfanityErrorType::Pronouns => "pronouns",
                    }
                ),
            ),

            _ => Status::InternalServerError.respond_to(req),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
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

    // other stuff
    pub(crate) color1: Option<i32>,
    pub(crate) color2: Option<i32>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DBUser {
    pub(crate) id: i64,
    pub(crate) token: Option<String>,
    pub(crate) token_expiration: Option<i64>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Challenge {
    pub(crate) bot_account_id: u32,
    pub(crate) challenge: String,
    pub(crate) id: Uuid,
}

#[derive(Debug)]
pub struct GDMessage {
    pub(crate) id: String, // this is an int but this makes it easier to use
    pub(crate) from: i64,
    pub(crate) subject: String,
}
