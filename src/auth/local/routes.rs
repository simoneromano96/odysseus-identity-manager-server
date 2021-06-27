use crate::{
	auth::AuthErrors,
	settings::{HANDLEBARS, SMTP_CLIENT},
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
			let html_mail = HANDLEBARS.render("signup", &user).unwrap();
			println!("{:?}", html_mail);

			let email = EmailBuilder::new()
				// Addresses can be specified by the tuple (email, alias)
				.to((&user.email, &user.preferred_username))
				// ... or by an address only
				.from("me@simoneromano.eu")
				.subject("You signed up successfully!")
				.html(html_mail)
				.build()
				.unwrap();
			let mut mailer = SmtpTransport::new(SMTP_CLIENT.clone());
			// Send the email
			let _result = mailer.send(email.into()).unwrap();

			Ok(Json(user.into()))
		}
		Err(e) => Err(AuthErrors::UserCreationError(UserErrors::ValidationError(e))),
	}
}
