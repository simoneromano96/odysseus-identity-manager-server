use log::error;
use ory_hydra_client::{
	apis::admin_api,
	models::{AcceptConsentRequest, CompletedRequest, ConsentRequest, ConsentRequestSession},
};
use wither::{bson::oid::ObjectId, mongodb::Database as MongoDatabase};

use crate::{settings::ORY_HYDRA_CONFIGURATION, user::User};

use super::ConsentErrors;

pub async fn create_user_session(subject: &str, db: &MongoDatabase) -> Result<ConsentRequestSession, ConsentErrors> {
	let id = ObjectId::with_string(subject).unwrap();
	let user = User::find_by_id(db, &id).await?.ok_or(ConsentErrors::UserNotFound)?;
	let session = ConsentRequestSession {
		id_token: Some(serde_json::to_value(&user)?),
		access_token: Some(serde_json::to_value(&user)?),
	};
	Ok(session)
}

pub async fn handle_accept_consent_request(
	subject: &str,
	db: &MongoDatabase,
	ask_consent_request: &ConsentRequest,
	scopes: &[String],
	consent_challenge: &str,
) -> Result<CompletedRequest, ConsentErrors> {
	let mut body = AcceptConsentRequest::new();
	body.grant_access_token_audience = ask_consent_request.requested_access_token_audience.clone();
	body.grant_scope = Some(scopes.to_vec());
	let session = create_user_session(subject, db).await?;
	body.session = Some(Box::new(session.clone()));
	body.remember = Some(true);
	body.remember_for = Some(0);
	let accept_consent_request: CompletedRequest =
		admin_api::accept_consent_request(&ORY_HYDRA_CONFIGURATION, &consent_challenge, Some(body))
			.await
			.map_err(|e| {
				error!("{:?}", e);
				ConsentErrors::HydraError
			})?;
	Ok(accept_consent_request)
}
