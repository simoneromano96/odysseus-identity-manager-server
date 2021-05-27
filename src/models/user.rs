use paperclip::actix::Apiv2Schema;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use wither::{
	bson::{doc, oid::ObjectId},
	mongodb::Database,
	prelude::*,
	WitherError,
};

use crate::utils::{hash_password, PasswordErrors};

/// User representation
#[derive(Debug, Model, Serialize, Deserialize)]
#[model(index(keys = r#"doc!{"username": 1}"#, options = r#"doc!{"unique": true}"#))]
pub struct User {
	/// The ID of the model.
	#[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
	pub id: Option<ObjectId>,
	/// The username.
	pub username: String,
	/// The hashed password.
	pub password: String,
}

#[derive(Error, Debug)]
pub enum UserErrors {
	#[error("{0}")]
	DatabaseError(#[from] WitherError),
	#[error("{0}")]
	HashError(#[from] PasswordErrors),
}

impl User {
	/// Create a new user
	pub async fn create_user(db: &Database, input: UserInput) -> Result<Self, UserErrors> {
		let UserInput { username, password } = input;

		// Hash the password
		let password = hash_password(&password)?;

		let mut user = User {
			id: None,
			username: String::from(username),
			password: String::from(password),
		};

		user.save(db, None).await?;

		Ok(user)
	}

	pub async fn find_by_id(db: &Database, id: &ObjectId) -> Result<Option<Self>, WitherError> {
		User::find_one(&db, doc! { "_id": id }, None).await
	}

	pub async fn find_by_username(db: &Database, username: &str) -> Result<Option<Self>, WitherError> {
		User::find_one(&db, doc! { "username": username }, None).await
	}
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
		UserInfo {
			id,
			username,
		}
	}
}

/// New user input data
#[derive(Debug, Serialize, Deserialize, Apiv2Schema)]
pub struct UserInput {
	/// The new user username, must be unique.
	pub username: String,
	/// The new user password.
	pub password: String,
	// User email
	// email: String,
}
