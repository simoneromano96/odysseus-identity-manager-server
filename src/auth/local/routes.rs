use crate::{
	auth::AuthErrors,
	settings::{APP_SETTINGS, HANDLEBARS, SMTP_CLIENT},
	user::{CreateUserInput, User, UserErrors, UserInfo},
};

use lettre::{SmtpTransport, Transport};
use lettre_email::EmailBuilder;
use paperclip::actix::{
	api_v2_operation, post,
	web::{Data, Json},
};
use validator::Validate;
use wither::mongodb::Database as MongoDatabase;

struct SignupEMailData {
	pub preferred_username: String,
	pub verification_code: String,
}

/// User signup
///
/// Creates a new user but doesn't log in the user
#[api_v2_operation]
#[post("/signup")]
pub async fn signup(
	db: Data<MongoDatabase>,
	create_user_input: Json<CreateUserInput>,
) -> Result<Json<UserInfo>, AuthErrors> {
	match create_user_input.validate() {
		Ok(_) => {
			// Create a user
			let user = User::create_user(&db, create_user_input.into_inner()).await?;
			let html_mail = HANDLEBARS.render("signup", &user)?;

			// Create email
			let email = EmailBuilder::new()
				// Destination address/alias
				.to((&user.email, &user.preferred_username))
				// Sender address/alias
				.from((&APP_SETTINGS.smtp.address, &APP_SETTINGS.smtp.alias))
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
