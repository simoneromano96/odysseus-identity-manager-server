use actix_session::Session;
use serde::{Deserialize, Serialize};
use wither::{
	bson::{doc, oid::ObjectId},
	mongodb::Database,
	prelude::*,
	WitherError,
};

use crate::{
	auth::NewUserInput,
	settings::init_keyed_totp_long,
	utils::{hash_password, verify_password},
};

use super::{AddressScope, EmailScope, PhoneScope, ProfileScope, UserErrors};

/// User representation
#[derive(Debug, Default, Model, Serialize, Deserialize)]
#[model(index(keys = r#"doc!{"email": 1}"#, options = r#"doc!{"unique": true}"#))]
pub struct User {
	/// The ID of the model and the Subject: Identifier for the End-User at the Issuer.
	#[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
	pub id: Option<ObjectId>,
	/// The user's hashed password.
	pub password: String,
	/// OpenID Connect Email scope
	#[serde(flatten)]
	pub email_scope: EmailScope,
	/// OpenID Connect Profile scope
	#[serde(flatten)]
	pub profile_scope: ProfileScope,
	/// OpenID Connect phone scope
	#[serde(flatten)]
	pub phone_scope: PhoneScope,
	/// OpenID Connect address scope
	pub address: Option<AddressScope>,
}

impl User {
	/// Create a new user
	pub async fn create_user(db: &Database, input: NewUserInput) -> Result<Self, UserErrors> {
		let NewUserInput {
			username,
			password,
			email,
		} = input;

		// Hash the password
		let password = hash_password(&password)?;

		let email_scope = EmailScope {
			email,
			..Default::default()
		};

		let profile_scope = ProfileScope {
			preferred_username: username,
			..Default::default()
		};

		let mut user = User {
			id: None,
			profile_scope,
			password,
			email_scope,
			..Default::default()
		};

		user.save(db, None).await?;

		Ok(user)
	}

	pub async fn login(db: &Database, email: &str, password: &str) -> Result<Self, UserErrors> {
		// Find the user
		let user = Self::find_by_email(db, email).await?.ok_or(UserErrors::UserNotFound)?;

		// Verify the password
		verify_password(&user.password, password)?;

		Ok(user)
	}

	pub async fn login_with_session(
		db: &Database,
		session: &Session,
		email: &str,
		password: &str,
	) -> Result<Self, UserErrors> {
		// Find the user
		let user = Self::find_by_email(db, email).await?.ok_or(UserErrors::UserNotFound)?;

		// Verify the password
		verify_password(&user.password, password)?;

		session.set("user_id", user.id.clone().unwrap().to_hex())?;

		Ok(user)
	}

	pub async fn user_from_session(db: &Database, session: &Session) -> Result<Self, UserErrors> {
		let user_id: String = session.get("user_id")?.ok_or(UserErrors::UserNotFound)?;
		let id = ObjectId::with_string(&user_id)?;
		Ok(Self::find_by_id(db, &id).await?.ok_or(UserErrors::UserNotFound)?)
	}

	pub async fn validate_email(&mut self, db: &Database, code: &str) -> Result<(), UserErrors> {
		// Check valid code with generator
		let generator = init_keyed_totp_long(&self.id.clone().unwrap().to_hex());
		let valid = generator.is_valid(code);
		if valid {
			// Change `user.email_verified` to `true` and persist the user
			self.email_scope.email_verified = true;
			self.save(db, None).await?;
			Ok(())
		} else {
			Err(UserErrors::InvalidCode)
		}
	}

	pub async fn find_by_id(db: &Database, id: &ObjectId) -> Result<Option<Self>, WitherError> {
		User::find_one(db, doc! { "_id": id }, None).await
	}

	pub async fn find_by_username(db: &Database, username: &str) -> Result<Option<Self>, WitherError> {
		User::find_one(db, doc! { "username": username }, None).await
	}

	pub async fn find_by_email(db: &Database, email: &str) -> Result<Option<Self>, WitherError> {
		User::find_one(db, doc! { "email": email }, None).await
	}
}
