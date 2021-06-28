use paperclip::actix::Apiv2Schema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Apiv2Schema)]
/// Consent request challenge
pub struct OAuthConsentRequest {
	/// The challenge code
	pub consent_challenge: String,
}

#[derive(Debug, Serialize, Deserialize, Apiv2Schema)]
pub struct ConsentQueryParams {
	pub consent_challenge: String,
	pub client_name: String,
	pub subject: String,
	pub requested_scope: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Apiv2Schema)]
#[serde(rename_all = "camelCase")]
pub struct OAuthConsentBody {
	/// The user authorized these scopes
	pub scopes: Vec<String>,
}
