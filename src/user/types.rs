use paperclip::actix::Apiv2Schema;
use serde::{Deserialize, Serialize};
use wither::bson::oid::ObjectId;

use super::User;

/// New user input data
#[derive(Debug, Serialize, Deserialize, Apiv2Schema)]
pub struct CreateUserInput {
	/// The new user username, must be unique.
	pub username: String,
	/// The new user password.
	pub password: String,
	// User email
	// email: String,
}

/// Available User info
#[derive(Debug, Serialize, Deserialize, Apiv2Schema)]
pub struct UserInfo {
	/// The ID of the user.
	#[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
	pub id: Option<ObjectId>,
	/// The username.
	pub username: String,
}

impl From<User> for UserInfo {
	fn from(user: User) -> Self {
		let User { id, username, .. } = user;
		UserInfo { id, username }
	}
}
