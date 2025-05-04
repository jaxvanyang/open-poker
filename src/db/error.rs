use std::fmt::Display;

use actix_web::HttpResponse;
use serde_json::json;

#[derive(Debug)]
pub enum Error {
	InternalServerError(String),
}

impl Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::InternalServerError(msg) => write!(f, "{msg}"),
		}
	}
}

impl std::error::Error for Error {}

impl actix_web::ResponseError for Error {
	fn error_response(&self) -> actix_web::HttpResponse<awc::body::BoxBody> {
		HttpResponse::build(self.status_code()).json(json!({
			"error": self.to_string()
		}))
	}
}

pub fn internal_server_error(msg: impl Display) -> Error {
	Error::InternalServerError(msg.to_string())
}
