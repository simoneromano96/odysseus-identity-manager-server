use crate::{auth::ConsentQueryParams, settings::ORY_HYDRA_CONFIGURATION, user::User};

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
use serde_json;
use serde_qs;
use url::Url;
use wither::{bson::oid::ObjectId, mongodb::Database as MongoDatabase};

use super::{ConsentErrors, OauthConsentBody, OauthConsentRequest};

/// User login
///
/// Starts the consent flow, responds with a redirect
#[api_v2_operation]
#[get("/consent")]
pub async fn get_consent(oauth_request: Query<OauthConsentRequest>) -> Result<HttpResponse, ConsentErrors> {
	info!("GET Consent request");

	let consent_challenge = oauth_request.into_inner().challenge;

	let ask_consent_request = admin_api::get_consent_request(&ORY_HYDRA_CONFIGURATION, &consent_challenge)
		.await
		.map_err(|e| {
			error!("{:?}", e);
			ConsentErrors::HydraError
		})?;

	let subject = ask_consent_request.subject.unwrap_or("".to_string());
	let client_name = if let Some(client) = ask_consent_request.client {
		client.client_name.unwrap_or("".to_string())
	} else {
		"".to_string()
	};

	info!("{:?}", &ask_consent_request.requested_scope);

	let query_params = ConsentQueryParams {
		challenge: consent_challenge.clone(),
		client_name,
		subject,
		requested_scope: ask_consent_request.requested_scope.unwrap_or(vec![]).clone(),
	};

	let mut redirect_to: Url = Url::parse("http://localhost:3000/consent")?;

	redirect_to.set_query(Some(&serde_qs::to_string(&query_params)?));

	info!("{:?}", redirect_to);

	// User is has already given consent
	if ask_consent_request.skip.unwrap_or(false) {
		info!("User has already given consent");
		let mut body = AcceptConsentRequest::new();
		body.grant_access_token_audience = ask_consent_request.requested_access_token_audience;

		let accept_consent_request =
			admin_api::accept_consent_request(&ORY_HYDRA_CONFIGURATION, &consent_challenge, Some(body))
				.await
				.map_err(|e| {
					error!("{:?}", e);
					ConsentErrors::HydraError
				})?;

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
	let consent_challenge = oauth_request.into_inner().challenge;

	let ask_consent_request: ConsentRequest =
		admin_api::get_consent_request(&ORY_HYDRA_CONFIGURATION, &consent_challenge)
			.await
			.map_err(|e| {
				error!("{:?}", e);
				ConsentErrors::HydraError
			})?;

	let subject: String = ask_consent_request.subject.ok_or(ConsentErrors::InvalidCookie)?;
	info!("{:?}", &subject);
	let id = ObjectId::with_string(&subject).unwrap();
	info!("{:?}", &id);
	let user = User::find_by_id(&db, &id).await?.ok_or(ConsentErrors::UserNotFound)?;
	info!("{:?}", &user);

	let session: ConsentRequestSession = ConsentRequestSession {
		id_token: Some(serde_json::to_value(&user)?),
		access_token: Some(serde_json::to_value(&user)?),
	};
	info!("{:?}", &session);

	let mut body = AcceptConsentRequest::new();
	body.grant_access_token_audience = ask_consent_request.requested_access_token_audience;
	body.grant_scope = Some(data.scopes.clone());
	body.session = Some(Box::new(session));
	body.remember = Some(true);
	body.remember_for = Some(0);

	let accept_consent_request: CompletedRequest =
		admin_api::accept_consent_request(&ORY_HYDRA_CONFIGURATION, &consent_challenge, Some(body))
			.await
			.map_err(|e| {
				error!("{:?}", e);
				ConsentErrors::HydraError
			})?;

	Ok(Json(accept_consent_request))
}
