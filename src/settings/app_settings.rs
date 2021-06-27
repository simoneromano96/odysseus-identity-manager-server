use std::{env, path::PathBuf};

use config::{Config, Environment, File};
use handlebars::Handlebars;
use lettre::{
	smtp::{authentication::Credentials, ConnectionReuseParameters},
	SmtpClient,
};
use log::info;
use once_cell::sync::Lazy;
use ory_hydra_client::apis::configuration::Configuration as OryConfiguration;
// use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::{HydraSettings, LoggerSettings, MongoSettings, ServerSettings, SMTPSettings};

pub static APP_SETTINGS: Lazy<Settings> = Lazy::new(Settings::init_config);
pub static ORY_HYDRA_CONFIGURATION: Lazy<OryConfiguration> = Lazy::new(init_ory_config);
pub static HANDLEBARS: Lazy<Handlebars> = Lazy::new(init_handlebars);
pub static SMTP_CLIENT: Lazy<SmtpClient> = Lazy::new(init_smtp);

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
pub struct TemplateSettings {
	/// The base path to the template directory
	pub path: String,
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
	/// Template directory
	pub template: TemplateSettings,
	/// SMTP configuration
	pub smtp: SMTPSettings,
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

		info!("APP CONFIGURATION: {:?}", r);

		r
	}
}

fn init_ory_config() -> OryConfiguration {
	use base64::encode as b64encode;
	use reqwest::header;

	let mut configuration = OryConfiguration::new();
	configuration.base_path = APP_SETTINGS.hydra.uri.clone();

	// Setup reqwest client
	let mut headers = header::HeaderMap::new();
	let credentials = b64encode(format!(
		"{}:{}",
		&APP_SETTINGS.hydra.username, &APP_SETTINGS.hydra.password,
	));
	let basic_auth = format!("Basic {}", credentials);

	headers.insert(
		header::AUTHORIZATION,
		header::HeaderValue::from_str(&basic_auth).expect("Could not create basic authorization header"),
	);

	let client = reqwest::Client::builder()
		.default_headers(headers)
		.build()
		.expect("Could not create client");

	// This does not work...
	// configuration.basic_auth = Some((
	// 	APP_SETTINGS.hydra.username.clone(),
	// 	Some(APP_SETTINGS.hydra.password.clone()),
	// ));

	configuration.client = client;

	info!("ORY CONFIGURATION: {:?}", configuration);

	configuration
}

fn init_handlebars() -> Handlebars<'static> {
	let mut base_path: PathBuf = PathBuf::new();
	base_path.push(&APP_SETTINGS.template.path);
	let signup_path = base_path.join("signup.hbs");
	// create the handlebars registry
	let mut handlebars = handlebars::Handlebars::new();
	// register template from a file and assign a name to it
	handlebars
		.register_template_file("signup", signup_path)
		.expect("Could not register `signup` template!");

	handlebars
}

fn init_smtp() -> SmtpClient {
	SmtpClient::new_simple("mail.simoneromano.eu")
		.unwrap()
		.credentials(Credentials::new("me@simoneromano.eu".into(), "djWfbMom".into()))
		.smtp_utf8(true)
		.connection_reuse(ConnectionReuseParameters::ReuseUnlimited)
}
