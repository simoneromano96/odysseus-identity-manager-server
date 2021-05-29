use paperclip::actix::Apiv2Schema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Apiv2Schema)]
pub struct OauthConsentRequest {
	pub challenge: String,
}


#[derive(Debug, Serialize, Deserialize, Apiv2Schema)]
pub struct ConsentQueryParams {
	pub challenge: String,
	pub client_name: String,
	pub subject: String,
	pub requested_scope: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Apiv2Schema)]
#[serde(rename_all = "camelCase")]
pub struct OauthConsentBody {
	pub scopes: Vec<String>,
}

