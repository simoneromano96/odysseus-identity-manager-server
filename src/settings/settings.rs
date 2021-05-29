use std::env;

use config::{Config, Environment, File};
use log::info;
use once_cell::sync::Lazy;
use ory_hydra_client::apis::configuration::Configuration as OryConfiguration;
// use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::{HydraSettings, LoggerSettings, MongoSettings, ServerSettings};

pub static APP_SETTINGS: Lazy<Settings> = Lazy::new(Settings::init_config);

pub static ORY_HYDRA_CONFIGURATION: Lazy<OryConfiguration> = Lazy::new(init_ory_config);

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RedisSettings {
	/// Redis client connection uri
	pub uri: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionSettings {
	/// Encryption secret
	pub secret: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
	/// Logger configuration
	pub logger: LoggerSettings,
	/// ORY Hydra client configuration
	pub hydra: HydraSettings,
	/// Mongo database configuration
	pub mongo: MongoSettings,
	/// Redis configuration
	pub redis: RedisSettings,
	/// HTTP server and app configuration
	pub server: ServerSettings,
	/// Session configuration
	pub session: SessionSettings,
}

impl Settings {
	fn init_config() -> Self {
		// Start config
		let mut s = Config::default();

		// Create a path
		let mut config_file_path = env::current_dir().expect("Cannot get current path");

		// Get current RUN_MODE, should be: development/production
		let current_env = env::var("RUN_MODE").unwrap_or_else(|_| String::from("development"));

		// From current path add /environments
		config_file_path.push("environments");
		// Add RUN_MODE.yaml
		config_file_path.push(format!("{}.yaml", current_env));

		// Add in the current environment file
		// Default to 'development' env
		s.merge(File::from(config_file_path).required(false))
			.expect("Could not read file");

		// Add in settings from the environment
		// ex. APP_DEBUG=1 sets debug key, APP_DATABASE_URL sets database.url key
		s.merge(Environment::new().prefix("APP").separator("_"))
			.expect("Cannot get env");

		// Deserialize configuration
		let r: Settings = s.try_into().expect("Configuration error");

		info!("{:?}", r);

		r
	}
}

fn init_ory_config() -> OryConfiguration {
	let mut configuration = OryConfiguration::new();
	configuration.base_path = APP_SETTINGS.hydra.url.clone();
	// configuration.client = Client::new();
	info!("{:?}", configuration);
	configuration
}
