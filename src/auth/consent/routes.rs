use crate::{settings::ORY_HYDRA_CONFIGURATION, user::User, utils::verify_password};

use actix_session::Session;
use actix_web::web::Query;
use log::{error, info};
use ory_hydra_client::{
	apis::admin_api,
	models::{AcceptConsentRequest, AcceptLoginRequest, CompletedRequest, ConsentRequest, LoginRequest},
};
use paperclip::actix::{
	api_v2_operation, get, post,
	web::{Data, HttpResponse, Json},
};
use url::Url;
use wither::mongodb::Database as MongoDatabase;

use super::{ConsentErrors, OauthConsentRequest};

/// User login
///
/// Starts the login flow, responds with a redirect
#[api_v2_operation]
#[get("/consent")]
pub async fn get_consent(oauth_request: Query<OauthConsentRequest>) -> Result<HttpResponse, ConsentErrors> {
	info!("GET Consent request");

	let consent_challenge = oauth_request.into_inner().consent_challenge;

	let ask_consent_request: ConsentRequest =
		admin_api::get_consent_request(&ORY_HYDRA_CONFIGURATION, &consent_challenge)
			.await
			.map_err(|e| {
				error!("{:?}", e);
				ConsentErrors::HydraError
			})?;

	let mut redirect_to: Url = Url::parse_with_params(
		"http://localhost:3000/consent",
		&[
			("challenge", consent_challenge.clone()),
			("client_name", ask_consent_request.client.unwrap().client_name.unwrap()),
			(
				"requested_scope",
				serde_qs::to_string(&ask_consent_request.requested_scope)?,
			),
		],
	)?;

	info!("{:?}", redirect_to);

	// User is has already given consent
	if ask_consent_request.skip.unwrap_or(false) {
		info!("User has already given consent");
		let subject = ask_consent_request.subject;
		let body = Some(AcceptConsentRequest::new());

		let accept_consent_request: CompletedRequest =
			admin_api::accept_consent_request(&ORY_HYDRA_CONFIGURATION, &consent_challenge, body)
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
/*
#[api_v2_operation]
#[post("/consent")]
pub async fn post_consent(
	login_input: Json<LoginInput>,
	oauth_request: Query<OauthConsentRequest>,
	session: Session,
	db: Data<MongoDatabase>,
) -> Result<Json<CompletedRequest>, ConsentErrors> {
	let login_challenge = oauth_request
		.into_inner()
		.login_challenge
		.ok_or(ConsentErrors::MissingLoginChallenge)?;

	let user = match session.get("user_id")? {
		Some(user_id) => {
			// We can be sure that the user exists if there is a session, unless the cookie has been revoked
			let user = User::find_by_id(&db, &user_id)
				.await?
				.ok_or(ConsentErrors::InvalidCookie)?;
			// Renew the session
			session.renew();

			user
		}
		None => {
			// Find the user
			let user = User::find_by_username(&db, &login_input.username)
				.await?
				.ok_or(ConsentErrors::UserNotFound)?;

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
			ConsentErrors::HydraError
		})?;

	info!("Hydra login response {:?}", login_request);

	Ok(Json(login_request))
}
*/
