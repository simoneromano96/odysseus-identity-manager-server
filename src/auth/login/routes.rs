use crate::{auth::handle_accept_login_request, settings::APP_SETTINGS, settings::ORY_HYDRA_CONFIGURATION, user::User, utils::verify_password};

use actix_session::Session;
use actix_web::web::Query;
use log::{error, info};
use ory_hydra_client::{
	apis::admin_api,
	models::{CompletedRequest, LoginRequest},
};
use paperclip::actix::{
	api_v2_operation, get, post,
	web::{Data, HttpResponse, Json},
};
use url::Url;
use wither::mongodb::Database as MongoDatabase;

use super::{LoginErrors, LoginInput, OauthLoginRequest};

/// User login
///
/// Starts the login flow, responds with a redirect
#[api_v2_operation]
#[get("/login")]
pub async fn get_login(oauth_request: Query<OauthLoginRequest>) -> Result<HttpResponse, LoginErrors> {
	info!("GET Login request");

	let login_challenge = oauth_request.into_inner().login_challenge;

	let ask_login_request: LoginRequest = admin_api::get_login_request(&ORY_HYDRA_CONFIGURATION, &login_challenge)
		.await
		.map_err(|e| {
			error!("{:?}", e);
			LoginErrors::HydraError
		})?;

	info!("{:?}", &ask_login_request);

	let mut redirect_to: Url = Url::parse(&APP_SETTINGS.server.clienturi)?;

	redirect_to.set_path("/login");
	redirect_to.set_query(Some(&format!("login_challenge={}", ask_login_request.challenge)));

	info!("{:?}", &redirect_to);
	info!("{:?}", &ask_login_request.client);

	// User is already authenticated
	if ask_login_request.skip {
		info!("User already authenticated");

		let subject = ask_login_request.subject;
		let accept_login_request = handle_accept_login_request(&subject, &login_challenge).await?;
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
) -> Result<Json<CompletedRequest>, LoginErrors> {
	let login_challenge = oauth_request.into_inner().login_challenge;

	let user = match session.get("user_id")? {
		Some(user_id) => {
			// We can be sure that the user exists if there is a session, unless the cookie has been revoked
			let user = User::find_by_id(&db, &user_id)
				.await?
				.ok_or(LoginErrors::InvalidCookie)?;
			// Renew the session
			session.renew();

			user
		}
		None => {
			// Find the user
			let user = User::find_by_username(&db, &login_input.username)
				.await?
				.ok_or(LoginErrors::UserNotFound)?;

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
	let subject = user.id.clone().unwrap().to_string();
	let accept_login_request = handle_accept_login_request(&subject, &login_challenge).await?;

	Ok(Json(accept_login_request))
}
