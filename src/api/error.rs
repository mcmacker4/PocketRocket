use crate::AnyError;
use rocket::http::ContentType;
use rocket::http::Status;
use rocket::request::Request;
use rocket::response::{self, Responder, Response};
use rocket::serde::{Deserialize, Serialize};
use serde_json::json;
use std::io::Cursor;

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiError {
    pub status: Status,
    pub message: String,
}

impl ApiError {
    pub fn new(status: Status, message: String) -> Self {
        Self { status, message }
    }
}

impl From<AnyError> for ApiError {
    fn from(err: AnyError) -> Self {
        ApiError {
            status: Status::InternalServerError,
            message: err.to_string(),
        }
    }
}

impl From<sqlx::Error> for ApiError {
    fn from(err: sqlx::Error) -> Self {
        ApiError {
            status: Status::InternalServerError,
            message: err.to_string(),
        }
    }
}

#[rocket::async_trait]
impl<'r> Responder<'r, 'static> for ApiError {
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'static> {
        let value = json!(
            {
                "status": self.status.code,
                "message": self.message,
            }
        );
        let msg = serde_json::to_string(&value).unwrap();

        Response::build()
            .status(self.status)
            .header(ContentType::JSON)
            .sized_body(msg.len(), Cursor::new(msg))
            .ok()
    }
}
