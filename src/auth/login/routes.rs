use crate::{
	auth::{handle_accept_login_request, AcceptedRequest},
	settings::APP_SETTINGS,
	settings::ORY_HYDRA_CONFIGURATION,
	user::User,
};

use actix_web::web::Query;
use log::error;
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
	db: Data<MongoDatabase>,
) -> Result<Json<AcceptedRequest>, LoginErrors> {
	let login_challenge = oauth_request.into_inner().login_challenge;

	let user = User::login(&db, &login_input.username, &login_input.password).await?;

	// Safe to unwrap since the user exists
	let subject = user.id.clone().unwrap().to_string();

	// Accept login request
	let accept_login_request = handle_accept_login_request(&subject, &login_challenge).await?;

	Ok(Json(accept_login_request.into()))
}
