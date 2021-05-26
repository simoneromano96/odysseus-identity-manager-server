use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct LoggerSettings {
	/// What should the (terminal) logger print
	pub level: String,
	/// File logger path output
	pub path: String,
}