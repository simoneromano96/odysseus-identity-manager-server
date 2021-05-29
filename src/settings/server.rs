use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// App and server configuration
pub struct ServerSettings {
	/// Client's uri
	pub clienturi: String,
	/// Server's port
	pub port: u16,
}
