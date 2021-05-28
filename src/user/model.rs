use serde::{Deserialize, Serialize};
use wither::{
	bson::{doc, oid::ObjectId},
	mongodb::Database,
	prelude::*,
	WitherError,
};

use crate::utils::{hash_password, verify_password};

use super::{CreateUserInput, UserErrors};

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

impl User {
	/// Create a new user
	pub async fn create_user(db: &Database, input: CreateUserInput) -> Result<Self, UserErrors> {
		let CreateUserInput { username, password } = input;

		// Hash the password
		let password = hash_password(&password)?;

		let mut user = User {
			id: None,
			username,
			password,
		};

		user.save(db, None).await?;

		Ok(user)
	}

	pub async fn login(db: &Database, username: &str, password: &str) -> Result<Self, UserErrors> {
		// Find the user
		let user = Self::find_by_username(&db, &username)
			.await?
			.ok_or(UserErrors::UserNotFound)?;

		// Verify the password
		verify_password(&user.password, &password)?;

		Ok(user)
	}

	pub async fn find_by_id(db: &Database, id: &ObjectId) -> Result<Option<Self>, WitherError> {
		User::find_one(&db, doc! { "_id": id }, None).await
	}

	pub async fn find_by_username(db: &Database, username: &str) -> Result<Option<Self>, WitherError> {
		User::find_one(&db, doc! { "username": username }, None).await
	}
}
