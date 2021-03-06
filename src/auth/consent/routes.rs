use crate::{
	auth::{handle_accept_consent_request, AcceptedRequest, ConsentQueryParams, Metadata},
	settings::{APP_SETTINGS, ORY_HYDRA_CONFIGURATION},
};

use log::{error, info};
use ory_hydra_client::{apis::admin_api, models::ConsentRequest};
use paperclip::actix::{
	api_v2_operation, get, post,
	web::{Data, HttpResponse, Json, Query},
};
use serde_qs;
use url::Url;
use wither::mongodb::Database as MongoDatabase;

use super::{ConsentErrors, OAuthConsentBody, OAuthConsentRequest};

/// OAUTH User get consent
///
/// Starts the consent request flow, responds with a redirect
#[api_v2_operation]
#[get("/consent")]
pub async fn get_consent(
	oauth_request: Query<OAuthConsentRequest>,
	db: Data<MongoDatabase>,
) -> Result<HttpResponse, ConsentErrors> {
	info!("Handling get consent request");

	let ask_consent_request = admin_api::get_consent_request(&ORY_HYDRA_CONFIGURATION, &oauth_request.consent_challenge)
		.await
		.map_err(|e| {
			error!("{:?}", e);
			ConsentErrors::HydraError
		})?;

	info!("{:?}", &ask_consent_request);

	let ConsentRequest {
		subject,
		client,
		requested_scope,
		challenge,
		..
	} = ask_consent_request.clone();

	let subject = subject.unwrap_or_default();
	let requested_scope = requested_scope.unwrap_or_default();

	let client_name = match &client {
		Some(client) => client.clone().client_name.unwrap_or_default(),
		None => "".to_string(),
	};

	// info!("{:?}", &redirect_to);

	let metadata = match client {
		Some(client) => {
			if let Some(metadata) = client.metadata {
				if let Ok(parsed_metadata) = serde_json::from_value(metadata) {
					parsed_metadata
				}
				Metadata::default()
			} else {
				Metadata::default()
			}
		}
		None => Metadata::default(),
	};

	let mut redirect_to;

	// If the oauth client is trusted or the User has already given consent, accept the consent
	if metadata.is_trusted || ask_consent_request.skip.unwrap_or(false) {
		let accept_consent_request = handle_accept_consent_request(
			&subject,
			&db,
			&ask_consent_request,
			&requested_scope,
			&oauth_request.consent_challenge,
		)
		.await?;

		redirect_to = Url::parse(&accept_consent_request.redirect_to)?;
	} else {
		let client_uri = Url::parse(&APP_SETTINGS.server.clienturi)?;
		redirect_to = client_uri.join("consent")?;

		let query_params = ConsentQueryParams {
			consent_challenge: challenge,
			client_name,
			subject: subject.clone(),
			requested_scope: requested_scope.clone(),
		};
		redirect_to.set_query(Some(&serde_qs::to_string(&query_params)?));
	}

	Ok(
		HttpResponse::PermanentRedirect()
			.header("Location", redirect_to.as_str())
			.finish(),
	)
}

/// OAUTH User post consent
///
/// Accepts the consent request, responds with a redirect to follow
#[api_v2_operation]
#[post("/consent")]
pub async fn post_consent(
	oauth_request: Query<OAuthConsentRequest>,
	data: Json<OAuthConsentBody>,
	db: Data<MongoDatabase>,
	// session: Session,
) -> Result<Json<AcceptedRequest>, ConsentErrors> {
	info!("Handling post consent request");

	let consent_challenge = &oauth_request.consent_challenge;

	let ask_consent_request = admin_api::get_consent_request(&ORY_HYDRA_CONFIGURATION, consent_challenge)
		.await
		.map_err(|e| {
			error!("{:?}", e);
			ConsentErrors::HydraError
		})?;

	let subject = ask_consent_request
		.subject
		.as_ref()
		.ok_or(ConsentErrors::UserNotFound)?;

	let accept_consent_request =
		handle_accept_consent_request(subject, &db, &ask_consent_request, &data.scopes, consent_challenge).await?;

	info!("{:?}", &accept_consent_request);

	Ok(Json(accept_consent_request.into()))
}
