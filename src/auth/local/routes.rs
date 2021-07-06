use crate::{
	auth::{AuthErrors, LoginInput},
	settings::{init_keyed_totp_long, HANDLEBARS},
	user::{User, UserErrors, UserInfo},
};

use actix_session::Session;

use paperclip::actix::{
	api_v2_operation, get, post,
	web::{Data, Json, Query},
};
use serde::{Deserialize, Serialize};
use validator::Validate;
use wither::mongodb::Database as MongoDatabase;

use super::{send_email_to_user, NewUserInput, ValidateCode};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
struct SignupEMailData {
	pub username: String,
	pub code: String,
}

/// LOCAL User signup
///
/// Creates a new user but doesn't log in the user
#[api_v2_operation]
#[post("/signup")]
pub async fn signup(
	db: Data<MongoDatabase>,
	create_user_input: Json<NewUserInput>,
) -> Result<Json<UserInfo>, AuthErrors> {
	match create_user_input.validate() {
		Ok(_) => {
			// Create a user
			let user = User::create_user(&db, create_user_input.into_inner()).await?;

			let username = user
				.profile_scope
				.preferred_username
				.clone()
				.unwrap_or_else(|| "Anonymous".to_string());

			// Safe to unwrap
			let user_id = user.id.clone().unwrap();
			let generator = init_keyed_totp_long(&user_id.to_hex());
			let code = generator.generate();

			let signup_data = SignupEMailData {
				username: username.clone(),
				code,
			};

			let html_mail = HANDLEBARS.render("signup", &signup_data)?;
			let email_title = "You signed up in Odysseus successfully!";

			send_email_to_user(&user.email_scope.email, &username, email_title, &html_mail)?;

			Ok(Json(user.into()))
		}
		Err(e) => Err(AuthErrors::UserError(UserErrors::ValidationError(e))),
	}
}

/// LOCAL User login
///
/// Logs in the user into Odysseus
#[api_v2_operation]
#[post("/login")]
pub async fn local_login(
	db: Data<MongoDatabase>,
	login_input: Json<LoginInput>,
	session: Session,
) -> Result<Json<UserInfo>, AuthErrors> {
	// Destructure login
	let LoginInput { email, password } = &login_input.into_inner();

	// Login the user, will also persist the session
	let user = User::login_with_session(&db, &session, email, password).await?;

	Ok(Json(user.into()))
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
struct ValidatedEMailData {
	pub username: String,
}

/// LOCAL User email validation
///
/// Validates user email, will set email_verified to true
#[api_v2_operation]
#[post("/validate-email")]
pub async fn validate_email(
	db: Data<MongoDatabase>,
	session: Session,
	code_input: Json<ValidateCode>,
) -> Result<Json<UserInfo>, AuthErrors> {
	match code_input.validate() {
		Ok(_) => {
			// Get user from session
			let mut user = User::user_from_session(&db, &session).await?;
			// Validate email
			user.validate_email(&db, &code_input.code).await?;

			let username = user
				.profile_scope
				.preferred_username
				.clone()
				.unwrap_or_else(|| "Anonymous".to_string());

			let validated_email_data = ValidatedEMailData {
				username: username.clone(),
			};
			// Send email
			let html_mail = HANDLEBARS.render("email-verified", &validated_email_data)?;
			let email_title = "You just verified your email successfully!";

			send_email_to_user(&user.email_scope.email, &username, email_title, &html_mail)?;
			// Send positive response
			Ok(Json(user.into()))
		}
		Err(e) => Err(AuthErrors::UserError(UserErrors::ValidationError(e))),
	}
}

/// LOCAL Current user info
///
/// Gets current user info from session
#[api_v2_operation]
#[get("/user-info")]
pub async fn user_info(db: Data<MongoDatabase>, session: Session) -> Result<Json<UserInfo>, AuthErrors> {
	// Get user from session
	let user = User::user_from_session(&db, &session).await?;
	Ok(Json(user.into()))
}
