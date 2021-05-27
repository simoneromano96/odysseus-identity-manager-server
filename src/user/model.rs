use serde::{Deserialize, Serialize};
use wither::{
	bson::{doc, oid::ObjectId},
	mongodb::Database,
	prelude::*,
	WitherError,
};

use crate::utils::hash_password;

use super::{UserErrors, UserInput};

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
