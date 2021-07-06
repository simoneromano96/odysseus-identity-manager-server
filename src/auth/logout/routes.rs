use crate::{auth::AcceptedRequest, settings::APP_SETTINGS, settings::ORY_HYDRA_CONFIGURATION};

use log::error;
use ory_hydra_client::apis::admin_api;
use paperclip::actix::{
	api_v2_operation, get, post,
	web::{HttpResponse, Json, Query},
};
use url::Url;

use super::{LogoutErrors, OauthLogoutRequest};

/// OAUTH get User logout
///
/// Starts the logout flow, responds with a redirect
#[api_v2_operation]
#[get("/logout")]
pub async fn get_logout(oauth_request: Query<OauthLogoutRequest>) -> Result<HttpResponse, LogoutErrors> {
	let ask_logout_request = admin_api::get_logout_request(&ORY_HYDRA_CONFIGURATION, &oauth_request.logout_challenge)
		.await
		.map_err(|e| {
			error!("{:?}", e);
			LogoutErrors::HydraError
		})?;

	let client_uri = Url::parse(&APP_SETTINGS.server.clienturi)?;

	let mut redirect_to = client_uri.join("login")?;
	redirect_to.set_query(Some(&format!(
		"logout_challenge={}",
		ask_logout_request.challenge.unwrap_or_default()
	)));

	Ok(
		HttpResponse::PermanentRedirect()
			.header("Location", redirect_to.as_str())
			.finish(),
	)
}

/// OAUTH post User logout
///
/// Logs out the user, responds with a redirect to follow
#[api_v2_operation]
#[post("/logout")]
pub async fn post_logout(oauth_request: Query<OauthLogoutRequest>) -> Result<Json<AcceptedRequest>, LogoutErrors> {
	let accept_logout_request =
		admin_api::accept_logout_request(&ORY_HYDRA_CONFIGURATION, &oauth_request.logout_challenge)
			.await
			.map_err(|e| {
				error!("{:?}", e);
				LogoutErrors::HydraError
			})?;

	Ok(Json(accept_logout_request.into()))
}
