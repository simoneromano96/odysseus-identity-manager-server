use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
/// SMTP configuration
pub struct SMTPSettings {
	/// The SMTP server domain/address
	pub domain: String,
	/// The username credential
	pub username: String,
	/// The password credential
	pub password: String,
}
