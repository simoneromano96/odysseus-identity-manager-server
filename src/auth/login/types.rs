use paperclip::actix::Apiv2Schema;
use serde::{Deserialize, Serialize};

/// New user input data
#[derive(Debug, Serialize, Deserialize, Apiv2Schema)]
#[serde(rename_all = "camelCase")]
pub struct LoginInput {
	/// The new user username, must be unique.
	pub username: String,
	/// The new user password.
	pub password: String,
}

#[derive(Debug, Serialize, Deserialize, Apiv2Schema)]
pub struct OauthLoginRequest {
	pub login_challenge: String,
}
