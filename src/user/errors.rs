use thiserror::Error;
use wither::WitherError;

use crate::utils::PasswordErrors;

#[derive(Error, Debug)]
/// Possible user errors
pub enum UserErrors {
	#[error("{0}")]
	DatabaseError(#[from] WitherError),
	#[error("{0}")]
	HashError(#[from] PasswordErrors),
}
