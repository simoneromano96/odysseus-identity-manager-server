use actix_web::Error as ActixError;
use thiserror::Error;
use validator::ValidationErrors;
use wither::{bson::oid::Error as ObjectIdError, WitherError};

use crate::utils::PasswordErrors;

#[derive(Error, Debug)]
/// Possible user errors
pub enum UserErrors {
	#[error("{0}")]
	DatabaseError(#[from] WitherError),
	#[error("{0}")]
	SessionError(#[from] ActixError),
	#[error("{0}")]
	HashError(#[from] PasswordErrors),
	#[error("User not found")]
	UserNotFound,
	#[error("User with email {0} was not found")]
	UserWithEmailNotFound(String),
	#[error("Invalid code")]
	InvalidCode,
	#[error("{0}")]
	ValidationError(#[from] ValidationErrors),
	#[error("{0}")]
	ObjectIdError(#[from] ObjectIdError),
}
