use actix_web::{http::StatusCode, Error as ActixError, HttpResponse, ResponseError};
use serde::{Deserialize, Serialize};
use serde_json::Error as JSONError;
use thiserror::Error;
use url::ParseError;
use wither::WitherError;

use crate::{user::UserErrors, utils::PasswordErrors};

#[derive(Debug, Deserialize, Serialize)]
struct ErrorResponse {
	error: String,
}

#[derive(Error, Debug)]
pub enum LogoutErrors {
	#[error("Internal server error")]
	ActixError(#[from] ActixError),
	#[error("Internal server error")]
	DatabaseError(#[from] WitherError),
	#[error("Could not create user")]
	UserCreationError(#[from] UserErrors),
	#[error("Password Error: {0}")]
	PasswordError(#[from] PasswordErrors),
	#[error("Ory hydra error")]
	HydraError,
	#[error("Invalid URL: {0}")]
	InvalidUrl(#[from] ParseError),
	#[error("Internal server error: {0}")]
	JSONParseError(#[from] JSONError),
}

impl ResponseError for LogoutErrors {
	fn error_response(&self) -> HttpResponse {
		let error_response = ErrorResponse {
			error: self.to_string(),
		};
		HttpResponse::build(self.status_code()).json(error_response)
	}

	fn status_code(&self) -> StatusCode {
		match self {
			Self::UserCreationError(UserErrors::DatabaseError(_)) => StatusCode::BAD_REQUEST,
			Self::PasswordError(_) => StatusCode::BAD_REQUEST,
			_ => StatusCode::INTERNAL_SERVER_ERROR,
		}
	}
}
