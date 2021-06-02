use crate::{
	auth::Metadata,
	auth::{handle_accept_login_request, AcceptedRequest},
	settings::APP_SETTINGS,
	settings::ORY_HYDRA_CONFIGURATION,
	user::User,
	utils::verify_password,
};

use actix_session::Session;
use actix_web::web::Query;
use log::{error};
use ory_hydra_client::{apis::admin_api, models::LoginRequest};
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
	let login_challenge = oauth_request.into_inner().login_challenge;

	let ask_login_request: LoginRequest = admin_api::get_login_request(&ORY_HYDRA_CONFIGURATION, &login_challenge)
		.await
		.map_err(|e| {
			error!("{:?}", e);
			LoginErrors::HydraError
		})?;

	let client_uri = Url::parse(&APP_SETTINGS.server.clienturi)?;

	let mut redirect_to = client_uri.join("login")?;
	redirect_to.set_query(Some(&format!("login_challenge={}", ask_login_request.challenge)));

	// info!("{:?}", &redirect_to);

	// User is already authenticated
	if ask_login_request.skip {
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
) -> Result<Json<AcceptedRequest>, LoginErrors> {
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
			let user = User::login(&db, &login_input.username, &login_input.password).await?;

			// Create a session for the user
			session.set("user_id", user.id.clone().unwrap())?;

			user
		}
	};

	let subject = user.id.clone().unwrap().to_string();
	let accept_login_request = handle_accept_login_request(&subject, &login_challenge).await?;

	// info!("{:?}", &accept_login_request);

	Ok(Json(accept_login_request.into()))
}
