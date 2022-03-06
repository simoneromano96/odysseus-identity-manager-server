use actix_web::{http::StatusCode, Error as ActixError, HttpResponse, ResponseError};
use paperclip::actix::api_v2_errors;
use serde::{Deserialize, Serialize};
use serde_json::Error as JsonError;
use serde_qs::Error as QSError;
use thiserror::Error;
use url::ParseError;
use wither::{bson::oid::Error as ObjectIdError, WitherError};

#[derive(Debug, Deserialize, Serialize)]
struct ErrorResponse {
	error: String,
}

#[api_v2_errors(
	code = 404,
	description = "Could not find user",
	code = 500,
	description = "There has been an error while validating this request, please try again"
)]
#[derive(Error, Debug)]
pub enum ConsentErrors {
	#[error("Internal server error")]
	ActixError(#[from] ActixError),
	#[error("Internal server error")]
	DatabaseError(#[from] WitherError),
	#[error("User not found")]
	UserNotFound,
	#[error("User is malformed: {0}")]
	UserMalformed(#[from] JsonError),
	#[error("Ory hydra error")]
	HydraError,
	#[error("Invalid URL: {0}")]
	InvalidUrl(#[from] ParseError),
	#[error("Invalid Query Parameters: {0}")]
	InvalidQueryParams(#[from] QSError),
	#[error("Invalid OID: {0}")]
	ObjectIdError(#[from] ObjectIdError),
}

impl ResponseError for ConsentErrors {
	fn error_response(&self) -> HttpResponse {
		let error_response = ErrorResponse {
			error: self.to_string(),
		};
		HttpResponse::build(self.status_code()).json(error_response)
	}

	fn status_code(&self) -> StatusCode {
		match self {
			Self::UserNotFound => StatusCode::NOT_FOUND,
			_ => StatusCode::INTERNAL_SERVER_ERROR,
		}
	}
}
