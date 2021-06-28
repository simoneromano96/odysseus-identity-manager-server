use paperclip::actix::Apiv2Schema;
use serde::{Deserialize, Serialize};
use validator::Validate;

/// New user input data
#[derive(Debug, Serialize, Deserialize, Apiv2Schema, Validate)]
pub struct NewUserInput {
	/// The new user username.
	pub username: Option<String>,
	/// The new user password.
	pub password: String,
	/// User email, must be unique.
	#[validate(email)]
	pub email: String,
}

/// Code validation input
#[derive(Debug, Serialize, Deserialize, Apiv2Schema, Validate)]
pub struct ValidateCode {
  /// The TOTP code.
  pub code: String,
}
