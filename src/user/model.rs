

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

use super::{Address, Gender, UserErrors};

/// User representation
#[derive(Debug, Default, Model, Serialize, Deserialize)]
#[model(index(keys = r#"doc!{"email": 1}"#, options = r#"doc!{"unique": true}"#))]
pub struct User {
	/// The ID of the model and the Subject: Identifier for the End-User at the Issuer.
	#[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
	pub id: Option<ObjectId>,
	/// Shorthand name by which the End-User wishes to be referred to at the RP, such as janedoe or j.doe. This value MAY be any valid JSON string including special characters such as @, /, or whitespace.
	pub preferred_username: Option<String>,
	/// The user's hashed password.
	pub password: String,
	/// End-User's preferred e-mail address. Its value MUST conform to the RFC 5322 [RFC5322] addr-spec syntax
	pub email: String,
	/// If the user email has been verified
	pub email_verified: bool,
	// The next data is all optional
	/// Given name(s) or first name(s) of the End-User. Note that in some cultures, people can have multiple given names; all can be present, with the names being separated by space characters.
	pub given_name: Option<String>,
	/// Middle name(s) of the End-User. Note that in some cultures, people can have multiple middle names; all can be present, with the names being separated by space characters. Also note that in some cultures, middle names are not used.
	pub middle_name: Option<String>,
	/// Surname(s) or last name(s) of the End-User. Note that in some cultures, people can have multiple family names or no family name; all can be present, with the names being separated by space characters.
	pub family_name: Option<String>,
	/// Casual name of the End-User that may or may not be the same as the given_name. For instance, a nickname value of Mike might be returned alongside a given_name value of Michael.
	pub nickname: Option<String>,
	/// URL of the End-User's profile page. The contents of this Web page SHOULD be about the End-User.
	pub profile: Option<String>,
	/// URL of the End-User's profile picture. This URL MUST refer to an image file (for example, a PNG, JPEG, or GIF image file), rather than to a Web page containing an image. Note that this URL SHOULD specifically reference a profile photo of the End-User suitable for displaying when describing the End-User, rather than an arbitrary photo taken by the End-User.
	pub picture: Option<String>,
	/// URL of the End-User's Web page or blog. This Web page SHOULD contain information published by the End-User or an organization that the End-User is affiliated with.
	pub website: Option<String>,
	/// End-User's gender. Values defined by this specification are female and male. Other values MAY be used when neither of the defined values are applicable.
	pub gender: Option<Gender>,
	/// End-User's birthday, represented as an ISO 8601:2004 [ISO8601‑2004] YYYY-MM-DD format.
	pub birthdate: Option<String>,
	/// String from zoneinfo [zoneinfo] time zone database representing the End-User's time zone. For example, Europe/Paris or America/Los_Angeles.
	pub zoneinfo: Option<String>,
	/// End-User's locale, represented as a BCP47 [RFC5646] language tag. This is typically an ISO 639-1 Alpha-2 [ISO639‑1] language code in lowercase and an ISO 3166-1 Alpha-2 [ISO3166‑1] country code in uppercase, separated by a dash. For example, en-US or fr-CA. As a compatibility note, some implementations have used an underscore as the separator rather than a dash, for example, en_US; Relying Parties MAY choose to accept this locale syntax as well.
	pub locale: Option<String>,
	/// End-User's preferred telephone number. E.164 [E.164] is RECOMMENDED as the format of this Claim, for example, +1 (425) 555-1212 or +56 (2) 687 2400. If the phone number contains an extension, it is RECOMMENDED that the extension be represented using the RFC 3966 [RFC3966] extension syntax, for example, +1 (604) 555-1234;ext=5678.
	pub phone_number: Option<String>,
	/// True if the End-User's phone number has been verified; otherwise false. When this Claim Value is true, this means that the OP took affirmative steps to ensure that this phone number was controlled by the End-User at the time the verification was performed. The means by which a phone number is verified is context-specific, and dependent upon the trust framework or contractual agreements within which the parties are operating. When true, the phone_number Claim MUST be in E.164 format and any extensions MUST be represented in RFC 3966 format.
	pub phone_number_verified: bool,
	/// End-User's preferred postal address. The value of the address member is a JSON [RFC4627] structure containing some or all of the members defined in Section 5.1.1.
	pub address: Option<Address>,
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

		let mut user = User {
			id: None,
			preferred_username: username,
			password,
			email,
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
			self.email_verified = true;
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
