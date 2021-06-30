use log::{error, info};
use ory_hydra_client::{
	apis::admin_api,
	models::{AcceptConsentRequest, CompletedRequest, ConsentRequest, ConsentRequestSession},
};
use wither::{bson::oid::ObjectId, mongodb::Database as MongoDatabase};

use crate::{
	settings::ORY_HYDRA_CONFIGURATION,
	user::{User, UserInfo},
};

use super::ConsentErrors;

pub async fn create_user_session(subject: &str, db: &MongoDatabase, scopes: &[String]) -> Result<ConsentRequestSession, ConsentErrors> {
	info!("Creating user session");
	let id = ObjectId::with_string(subject).unwrap();
	let user = User::find_by_id(db, &id).await?.ok_or(ConsentErrors::UserNotFound)?;
	let mut user_info = UserInfo::default();
	scopes.iter().for_each(|scope| {
		if scope == "email" {
			user_info.email_scope = Some(user.email_scope.clone());
		} else if scope == "profile" {
			user_info.profile_scope = Some(user.profile_scope.clone());
		} else if scope == "phone" {
			user_info.phone_scope = Some(user.phone_scope.clone());
		} else if scope == "address" {
			user_info.address = user.address.clone();
		}
	});
	let session = ConsentRequestSession {
		id_token: Some(serde_json::to_value(&user_info)?),
		access_token: Some(serde_json::to_value(&user_info)?),
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
	info!("Handling accept_consent_request");
	let mut body = AcceptConsentRequest::new();
	body.grant_access_token_audience = ask_consent_request.requested_access_token_audience.clone();
	body.grant_scope = Some(scopes.to_vec());
	let session = create_user_session(subject, db, scopes).await?;
	body.session = Some(Box::new(session.clone()));
	body.remember = Some(true);
	body.remember_for = Some(0);
	let accept_consent_request =
		admin_api::accept_consent_request(&ORY_HYDRA_CONFIGURATION, consent_challenge, Some(body))
			.await
			.map_err(|e| {
				error!("{:?}", e);
				ConsentErrors::HydraError
			})?;
	Ok(accept_consent_request)
}
