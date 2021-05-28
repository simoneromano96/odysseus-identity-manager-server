use crate::{
	auth::{AuthErrors, OAuthLoginInfo},
	settings::ORY_HYDRA_CONFIGURATION,
	user::{CreateUserInput, User, UserInfo},
	utils::verify_password,
};

use actix_session::Session;
use actix_web::web::Query;
use log::{error, info};
use ory_hydra_client::{
	apis::{
		admin_api::{self, AcceptConsentRequestError, AcceptLoginRequestError},
		configuration::Configuration,
		Error as OryError, ResponseContent,
	},
	models::{AcceptConsentRequest, AcceptLoginRequest, CompletedRequest, LoginRequest},
};

use paperclip::actix::{
	api_v2_operation, get, post,
	web::{Data, HttpResponse, Json},
};

use url::Url;
use wither::mongodb::Database as MongoDatabase;

use super::{LoginInput, OauthLoginRequest};

/// User login
///
/// Starts the login flow, responds with a redirect
#[api_v2_operation]
#[get("/login")]
pub async fn get_login(oauth_request: Query<OauthLoginRequest>) -> Result<HttpResponse, AuthErrors> {
	info!("GET Login request");

	let login_challenge = oauth_request
		.into_inner()
		.login_challenge
		.ok_or(AuthErrors::MissingLoginChallenge)?;

	let ask_login_request: LoginRequest = admin_api::get_login_request(&ORY_HYDRA_CONFIGURATION, &login_challenge)
		.await
		.map_err(|e| {
			error!("{:?}", e);
			AuthErrors::HydraError
		})?;

	let mut redirect_to: Url =
		Url::parse_with_params("http://localhost:3000/login", &[("login_challenge", &login_challenge)])?;

	// User is already authenticated
	if ask_login_request.skip {
		info!("User already authenticated");
		let subject = ask_login_request.subject;
		let body = Some(AcceptLoginRequest::new(subject));

		let accept_login_request: CompletedRequest =
			admin_api::accept_login_request(&ORY_HYDRA_CONFIGURATION, &login_challenge, body)
				.await
				.map_err(|e| {
					error!("{:?}", e);
					AuthErrors::HydraError
				})?;

		redirect_to = Url::parse(&accept_login_request.redirect_to)?;
	}

	Ok(
		HttpResponse::PermanentRedirect()
			.header("Location", redirect_to.as_str())
			.finish(),
	)
}

/// User login
///
/// Logs in the user via the provided credentials, responds with a redirect to follow
#[api_v2_operation]
#[post("/login")]
pub async fn post_login(
	login_input: Json<LoginInput>,
	oauth_request: Query<OauthLoginRequest>,
	session: Session,
	db: Data<MongoDatabase>,
) -> Result<Json<CompletedRequest>, AuthErrors> {
	let login_challenge = oauth_request
		.into_inner()
		.login_challenge
		.ok_or(AuthErrors::MissingLoginChallenge)?;

	let user = match session.get("user_id")? {
		Some(user_id) => {
			// We can be sure that the user exists if there is a session, unless the cookie has been revoked
			let user = User::find_by_id(&db, &user_id)
				.await?
				.ok_or(AuthErrors::InvalidCookie)?;
			// Renew the session
			session.renew();

			user
		}
		None => {
			// Find the user
			let user = User::find_by_username(&db, &login_input.username)
				.await?
				.ok_or(AuthErrors::UserNotFound)?;

			// Verify the password
			verify_password(&user.password, &login_input.password)?;

			info!("User logged in: {:?}", &user);

			// Create a session for the user
			session.set("user_id", user.id.clone().unwrap())?;

			user
		}
	};

	// let response;

	info!("Handling a login challenge");
	let body = Some(AcceptLoginRequest::new(user.id.clone().unwrap().to_string()));
	let login_request = admin_api::accept_login_request(&ORY_HYDRA_CONFIGURATION, &login_challenge, body)
		.await
		.map_err(|e| {
			error!("{:?}", e);
			AuthErrors::HydraError
		})?;

	info!("Hydra login response {:?}", login_request);

	Ok(Json(login_request))
}
