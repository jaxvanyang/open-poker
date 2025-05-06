use std::fmt::Display;

use actix_web::HttpResponse;
use awc::http::StatusCode;
use serde_json::json;

#[derive(Debug)]
pub enum ErrorType {
	InternalServerError,
	DatabaseError,
	UnauthorizedError,
}

impl Display for ErrorType {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			ErrorType::InternalServerError => write!(f, "internal server error"),
			ErrorType::DatabaseError => write!(f, "database error"),
			ErrorType::UnauthorizedError => write!(f, "unauthorized"),
		}
	}
}

#[derive(Debug)]
pub struct Error {
	pub r#type: ErrorType,
	pub msg: String,
}

pub type Result<T> = std::result::Result<T, Error>;

impl Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}: {}", self.r#type, self.msg)
	}
}

impl std::error::Error for Error {}

impl actix_web::ResponseError for Error {
	fn status_code(&self) -> StatusCode {
		match self.r#type {
			ErrorType::InternalServerError | ErrorType::DatabaseError => {
				StatusCode::INTERNAL_SERVER_ERROR
			}
			ErrorType::UnauthorizedError => StatusCode::UNAUTHORIZED,
		}
	}

	fn error_response(&self) -> actix_web::HttpResponse<awc::body::BoxBody> {
		let msg = if cfg!(debug_assertions) {
			&self.msg
		} else {
			match self.r#type {
				ErrorType::InternalServerError | ErrorType::UnauthorizedError => &self.msg,
				ErrorType::DatabaseError => "something went wrong, please retry",
			}
		};
		HttpResponse::build(self.status_code()).json(json!({
			"error": msg
		}))
	}
}

pub fn internal_server_error(msg: impl Display) -> Error {
	Error {
		r#type: ErrorType::InternalServerError,
		msg: msg.to_string(),
	}
}

pub fn unauthorized_error(msg: impl Display) -> Error {
	Error {
		r#type: ErrorType::UnauthorizedError,
		msg: msg.to_string(),
	}
}

impl From<rusqlite::Error> for Error {
	fn from(err: rusqlite::Error) -> Self {
		Self {
			r#type: ErrorType::DatabaseError,
			msg: err.to_string(),
		}
	}
}
