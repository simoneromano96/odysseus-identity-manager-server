use actix_web::{http::StatusCode, Error as ActixError, HttpResponse, ResponseError};
use handlebars::RenderError;
use lettre::{address::AddressError, error::Error as LettreError};
use paperclip::actix::api_v2_errors;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use url::ParseError;
use wither::WitherError;

use crate::{user::UserErrors, utils::PasswordErrors};

#[derive(Debug, Deserialize, Serialize)]
struct ErrorResponse {
	error: String,
}

// TODO: fix errors
#[api_v2_errors(
	code = 400,
	description = "Wrong input",
	code = 401,
	description = "Wrong password",
	code = 404,
	description = "User not found, probably wrong credentials",
	code = 500,
	description = "Internal server error, could be a db connection error, email server error"
)]
#[derive(Error, Debug)]
pub enum AuthErrors {
	#[error("Internal server error")]
	ActixError(#[from] ActixError),
	#[error("Internal server error")]
	DatabaseError(#[from] WitherError),
	#[error("{0}")]
	UserError(#[from] UserErrors),
	#[error("{0}")]
	PasswordError(#[from] PasswordErrors),
	#[error("Invalid URL: {0}")]
	InvalidUrl(#[from] ParseError),
	#[error("{0}")]
	HandlebarsError(#[from] RenderError),
	#[error("Email error: {0}")]
	EmailError(#[from] LettreError),
	#[error("Invalid address: {0}")]
	InvalidEmailAddress(#[from] AddressError),
	#[error("Could not send email!")]
	SendEmailError,
}

impl ResponseError for AuthErrors {
	fn error_response(&self) -> HttpResponse {
		let error_response = ErrorResponse {
			error: self.to_string(),
		};
		HttpResponse::build(self.status_code()).json(error_response)
	}

	fn status_code(&self) -> StatusCode {
		match self {
			Self::UserError(UserErrors::DatabaseError(_)) => StatusCode::BAD_REQUEST,
			Self::UserError(UserErrors::ValidationError(_)) => StatusCode::BAD_REQUEST,
			Self::UserError(UserErrors::UserNotFound) => StatusCode::NOT_FOUND,
			Self::UserError(UserErrors::UserWithEmailNotFound(_)) => StatusCode::NOT_FOUND,
			Self::UserError(UserErrors::HashError(PasswordErrors::InvalidPassword)) => StatusCode::UNAUTHORIZED,
			Self::PasswordError(_) => StatusCode::BAD_REQUEST,
			_ => StatusCode::INTERNAL_SERVER_ERROR,
		}
	}
}
