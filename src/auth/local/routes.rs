use crate::{
	auth::AuthErrors,
	settings::{init_keyed_totp_long, SMTPSettings, APP_SETTINGS, HANDLEBARS, SMTP_CLIENT},
	user::{User, UserErrors, UserInfo},
};

use actix_web::HttpResponse;
use lettre::{SmtpTransport, Transport};
use lettre_email::EmailBuilder;
use paperclip::actix::{
	api_v2_operation, get, post,
	web::{Data, Json, Query},
};
use serde::{Deserialize, Serialize};
use validator::Validate;
use wither::mongodb::Database as MongoDatabase;

use super::{NewUserInput, ValidateCode};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
struct SignupEMailData {
	pub username: String,
	pub code: String,
}

/// User signup
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

			let username = user.preferred_username.clone().unwrap_or("Anonymous".to_string());

			// Safe to unwrap
			let user_id = user.id.clone().unwrap();
			let generator = init_keyed_totp_long(&user_id.to_hex());
			let code = generator.generate();

			let signup_data = SignupEMailData {
				username: username.clone(),
				code,
			};

			let html_mail = HANDLEBARS.render("signup", &signup_data)?;
			let SMTPSettings { address, alias, .. } = &APP_SETTINGS.smtp;

			// Create email
			let email = EmailBuilder::new()
				// Destination address/alias
				.to((&user.email, &username))
				// Sender address/alias
				.from((address, alias))
				// Email subject
				.subject("You signed up successfully!")
				// Email html body
				.html(html_mail)
				.build()?;

			// Create transport
			let mut mailer = SmtpTransport::new(SMTP_CLIENT.clone());
			// Send the email
			mailer.send(email.into()).map_err(|_| AuthErrors::SendEmailError)?;

			Ok(Json(user.into()))
		}
		Err(e) => Err(AuthErrors::UserCreationError(UserErrors::ValidationError(e))),
	}
}

/// User email validation
///
/// Validates user email, will set email_verified to true
/// CURRENTLY UNIMPLENTED!
#[api_v2_operation]
#[get("/validate-email")]
pub async fn validate_email(_db: Data<MongoDatabase>, code: Query<ValidateCode>) -> Result<HttpResponse, AuthErrors> {
	match code.validate() {
		Ok(_) => {
			// Get user from session
			// let user_id = session.get()
			// let user = User::find_by_id(user_id)
			// Check valid code with generator
			// let generator = init_keyed_totp_long(&user_id.to_hex());
			// generator.is_valid(code);
			// Change `user.email_verified` to `true` and persist the user
			// user.email_verified = true
			// user.save().await?;
			// Send email
			// Send positive response
			Ok(HttpResponse::NotImplemented().body(""))
		}
		Err(e) => Err(AuthErrors::UserCreationError(UserErrors::ValidationError(e))),
	}
}
