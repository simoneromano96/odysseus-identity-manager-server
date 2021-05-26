use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Mongo database configuration
pub struct MongoSettings {
	/// DB Connection URI
	pub uri: String,
	/// DB Name
	pub database: String,
}
