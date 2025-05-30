use std::fmt::Display;

use actix_web::{HttpResponse, http::StatusCode};
use serde_json::json;

#[derive(Debug)]
pub enum ErrorType {
	InternalServerError,
	DatabaseError,
	UnauthorizedError,
	ConflictError,
	NotFoundError,
	BadRequestError,
	ForbiddenError,
}

impl Display for ErrorType {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			ErrorType::InternalServerError => write!(f, "internal server error"),
			ErrorType::DatabaseError => write!(f, "database error"),
			ErrorType::UnauthorizedError => write!(f, "unauthorized"),
			ErrorType::ConflictError => write!(f, "conflict"),
			ErrorType::NotFoundError => write!(f, "not found"),
			ErrorType::BadRequestError => write!(f, "bad request"),
			ErrorType::ForbiddenError => write!(f, "forbidden"),
		}
	}
}

/// Convience type for `actix_web::Error`
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
			ErrorType::ConflictError => StatusCode::CONFLICT,
			ErrorType::NotFoundError => StatusCode::NOT_FOUND,
			ErrorType::BadRequestError => StatusCode::BAD_REQUEST,
			ErrorType::ForbiddenError => StatusCode::FORBIDDEN,
		}
	}

	fn error_response(&self) -> actix_web::HttpResponse<actix_web::body::BoxBody> {
		let msg = if cfg!(debug_assertions) {
			&self.msg
		} else {
			match self.r#type {
				ErrorType::InternalServerError
				| ErrorType::UnauthorizedError
				| ErrorType::ConflictError
				| ErrorType::NotFoundError
				| ErrorType::BadRequestError
				| ErrorType::ForbiddenError => &self.msg,
				ErrorType::DatabaseError => "something went wrong, please retry",
			}
		};
		HttpResponse::build(self.status_code()).json(json!({
			"error": msg
		}))
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

pub fn conflict_error(msg: impl Display) -> Error {
	Error {
		r#type: ErrorType::ConflictError,
		msg: msg.to_string(),
	}
}

pub fn not_found_error(msg: impl Display) -> Error {
	Error {
		r#type: ErrorType::NotFoundError,
		msg: msg.to_string(),
	}
}

pub fn bad_request_error(msg: impl Display) -> Error {
	Error {
		r#type: ErrorType::BadRequestError,
		msg: msg.to_string(),
	}
}

pub fn forbidden_error(msg: impl Display) -> Error {
	Error {
		r#type: ErrorType::ForbiddenError,
		msg: msg.to_string(),
	}
}
