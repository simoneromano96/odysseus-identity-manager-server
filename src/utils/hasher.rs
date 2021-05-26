use argon2::{password_hash::SaltString, Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use once_cell::sync::Lazy;
use rand::rngs::OsRng;
use thiserror::Error;

static ARGON_2: Lazy<Argon2> = Lazy::new(|| Argon2::default());

#[derive(Error, Debug)]
pub enum PasswordErrors {
	#[error("Hashing error")]
	HashError,
	#[error("Invalid password")]
	InvalidPassword,
}

pub fn hash_password(password: &str) -> Result<String, PasswordErrors> {
	// Generate salt string from OS entropy
	let salt = SaltString::generate(&mut OsRng);

	// Hash password to PHC string ($argon2id$v=19$...)
	let password_hash = ARGON_2
		.hash_password_simple(password.as_bytes(), &salt)
		.map_err(|_| PasswordErrors::HashError)?
		.to_string();

	Ok(password_hash)
}

pub fn verify_password(password_hash: &str, password: &str) -> Result<(), PasswordErrors> {
	// Create Hash from PHC string
	let hash = PasswordHash::new(password_hash).map_err(|_| PasswordErrors::HashError)?;

	ARGON_2
		.verify_password(password.as_bytes(), &hash)
		.map_err(|_| PasswordErrors::InvalidPassword)
}
