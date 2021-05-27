use paperclip::actix::Apiv2Schema;
use serde::{Deserialize, Serialize};
use wither::bson::oid::ObjectId;

/// New user input data
#[derive(Debug, Serialize, Deserialize, Apiv2Schema)]
#[serde(rename_all = "camelCase")]
pub struct LoginInput {
	/// The new user username, must be unique.
	pub username: String,
	/// The new user password.
	pub password: String,
}

#[derive(Deserialize)]
pub struct OauthLoginRequest {
	pub login_challenge: Option<String>,
}
