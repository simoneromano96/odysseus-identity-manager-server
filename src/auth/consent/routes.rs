use crate::{
	auth::{create_user_session, handle_accept_consent_request, ConsentQueryParams},
	settings::{APP_SETTINGS, ORY_HYDRA_CONFIGURATION},
	user::User,
};

use actix_web::web::Query;
use log::{error, info};
use ory_hydra_client::{
	apis::admin_api,
	models::{
		AcceptConsentRequest, AcceptLoginRequest, CompletedRequest, ConsentRequest, ConsentRequestSession, LoginRequest,
	},
};
use paperclip::actix::{
	api_v2_operation, get, post,
	web::{Data, HttpResponse, Json},
};

use serde_qs;
use url::Url;
use wither::mongodb::Database as MongoDatabase;

use super::{ConsentErrors, OauthConsentBody, OauthConsentRequest};

/// User login
///
/// Starts the consent flow, responds with a redirect
#[api_v2_operation]
#[get("/consent")]
pub async fn get_consent(
	oauth_request: Query<OauthConsentRequest>,
	db: Data<MongoDatabase>,
) -> Result<HttpResponse, ConsentErrors> {
	info!("GET Consent request");

	let consent_challenge = oauth_request.into_inner().consent_challenge;

	let ask_consent_request = admin_api::get_consent_request(&ORY_HYDRA_CONFIGURATION, &consent_challenge)
		.await
		.map_err(|e| {
			error!("{:?}", e);
			ConsentErrors::HydraError
		})?;

	let ConsentRequest {
		subject,
		client,
		requested_scope,
		challenge,
		..
	} = ask_consent_request.clone();

	let subject = subject.unwrap_or_default();
	let requested_scope = requested_scope.unwrap_or_default();

	let client_name = match client {
		Some(client) => client.client_name.unwrap_or_default(),
		None => "".to_string(),
	};

	let query_params = ConsentQueryParams {
		consent_challenge: challenge,
		client_name,
		subject: subject.clone(),
		requested_scope: requested_scope.clone(),
	};

	let mut redirect_to: Url = Url::parse(&APP_SETTINGS.server.clienturi)?;

	redirect_to.set_path("/consent");
	redirect_to.set_query(Some(&serde_qs::to_string(&query_params)?));

	info!("{:?}", redirect_to);

	// User is has already given consent
	if ask_consent_request.skip.unwrap_or(false) {
		let accept_consent_request = handle_accept_consent_request(
			&subject,
			&db,
			&ask_consent_request,
			&requested_scope,
			&consent_challenge,
		)
		.await?;

		redirect_to = Url::parse(&accept_consent_request.redirect_to)?;
	}

	Ok(
		HttpResponse::PermanentRedirect()
			.header("Location", redirect_to.as_str())
			.finish(),
	)
}

// User login
//
// Logs in the user via the provided credentials, responds with a redirect to follow
#[api_v2_operation]
#[post("/consent")]
pub async fn post_consent(
	oauth_request: Query<OauthConsentRequest>,
	data: Json<OauthConsentBody>,
	db: Data<MongoDatabase>,
	// session: Session,
) -> Result<Json<CompletedRequest>, ConsentErrors> {
	let consent_challenge = oauth_request.into_inner().consent_challenge;

	let ask_consent_request: ConsentRequest =
		admin_api::get_consent_request(&ORY_HYDRA_CONFIGURATION, &consent_challenge)
			.await
			.map_err(|e| {
				error!("{:?}", e);
				ConsentErrors::HydraError
			})?;

	let subject = ask_consent_request
		.subject
		.as_ref()
		.ok_or(ConsentErrors::InvalidCookie)?;

	let accept_consent_request =
		handle_accept_consent_request(subject, &db, &ask_consent_request, &data.scopes, &consent_challenge).await?;

	Ok(Json(accept_consent_request))
}