use actix_web::{http::StatusCode, Error as ActixError, HttpResponse, ResponseError};
use paperclip::actix::api_v2_errors;
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
#[api_v2_errors(
	code=500,
)]
pub enum LogoutErrors {
	#[error("Internal server error")]
	ActixError(#[from] ActixError),
	#[error("Ory hydra error")]
	HydraError,
	#[error("Invalid URL: {0}")]
	InvalidUrl(#[from] ParseError),
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
			_ => StatusCode::INTERNAL_SERVER_ERROR,
		}
	}
}
