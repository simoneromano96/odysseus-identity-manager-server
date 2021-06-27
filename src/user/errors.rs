use thiserror::Error;
use validator::ValidationErrors;
use wither::WitherError;

use crate::utils::PasswordErrors;

#[derive(Error, Debug)]
/// Possible user errors
pub enum UserErrors {
	#[error("{0}")]
	DatabaseError(#[from] WitherError),
	#[error("{0}")]
	HashError(#[from] PasswordErrors),
	#[error("User not found")]
	UserNotFound,
	// #[error("Validation error")]
	// ValidationError,
	#[error("{0}")]
	ValidationError(#[from] ValidationErrors),
}
