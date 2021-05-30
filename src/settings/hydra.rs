use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Ory Hydra client configuration
pub struct HydraSettings {
	/// The url path for the ORY hydra server
	pub uri: String,
	/// The basic auth username
	pub username: String,
	/// The basic auth password
	pub password: String,
}
