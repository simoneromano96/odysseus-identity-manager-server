use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Ory Hydra client configuration
pub struct HydraSettings {
  /// The url path for the ORY hydra server
  pub url: String,
}