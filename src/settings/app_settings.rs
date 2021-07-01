use std::{env, path::PathBuf};

use config::{Config, Environment, File};
use handlebars::Handlebars;
use lettre::{transport::smtp::authentication::Credentials, SmtpTransport};
use libreauth::{
	hash::HashFunction::Sha3_512,
	oath::{TOTPBuilder, TOTP},
};
use log::info;
use once_cell::sync::Lazy;
use ory_hydra_client::apis::configuration::Configuration as OryConfiguration;
use serde::{Deserialize, Serialize};

use super::{HydraSettings, LoggerSettings, MongoSettings, SMTPSettings, ServerSettings};

pub static APP_SETTINGS: Lazy<Settings> = Lazy::new(Settings::init_config);
pub static ORY_HYDRA_CONFIGURATION: Lazy<OryConfiguration> = Lazy::new(init_ory_config);
pub static HANDLEBARS: Lazy<Handlebars> = Lazy::new(init_handlebars);
pub static SMTP_CLIENT: Lazy<SmtpTransport> = Lazy::new(init_smtp);
pub static SIMPLE_TOTP_LONG_GENERATOR: Lazy<TOTP> = Lazy::new(init_simple_totp_long);

// TODO: add const for template names

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
pub struct TOTPSettings {
	/// The token creation secret
	pub secret: String,
	/// The token valid period in seconds
	pub period: String,
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
	/// Template configuration
	pub template: TemplateSettings,
	/// SMTP configuration
	pub smtp: SMTPSettings,
	/// Time-based one time token password configuration
	pub totp: TOTPSettings,
}

impl Settings {
	fn init_config() -> Self {
		// Start config
		let mut s = Config::default();

		// Create a path
		let mut config_file_path = env::current_dir().expect("Cannot get current path");

		// Get current RUN_MODE, should be: development/production
		let current_env = env::var("RUN_MODE").unwrap_or_else(|_| String::from("development"));

		// Get current LOCAL, should be: true when running locally
		let local = env::var("LOCAL").unwrap_or_default() == "true";

		// From current path add /environments
		config_file_path.push("environments");

		// ex. development/production
		let mut filename = current_env;
		// Add local
		if local {
			filename.push_str(".local");
		}
		// Add extension
		filename.push_str(".yaml");

		// Add RUN_MODE{.local}.yaml
		config_file_path.push(filename);

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
	// Base path for all templates
	let mut base_path: PathBuf = PathBuf::new();
	base_path.push(&APP_SETTINGS.template.path);

	info!("Init template folder: {:?}", &base_path);

	// Create the handlebars registry
	let mut handlebars = handlebars::Handlebars::new();

	// Register signup template
	handlebars
		.register_template_file("signup", base_path.join("signup.hbs"))
		.expect("Could not register `signup` template!");

	// Register verified email template
	handlebars
		.register_template_file("email-verified", base_path.join("email-verified.hbs"))
		.expect("Could not register `email-verified` template!");

	info!("Successfully Registered all templates!");

	handlebars
}

fn init_smtp() -> SmtpTransport {
	let SMTPSettings {
		domain,
		username,
		password,
		..
	} = &APP_SETTINGS.smtp;

	info!(
		"Init smtp client: DOMAIN: {:?}, USERNAME: {:?}, PASSWORD: {:?}",
		&domain, &username, &password
	);

	SmtpTransport::relay(domain)
		.expect("Could not create SMTP transport!")
		.credentials(Credentials::new(username.clone(), password.clone()))
		.build()

	// SmtpClient::new_simple(domain)
	// 	.expect("Could not create SMTP client!")
	// 	.credentials(Credentials::new(username.clone(), password.clone()))
	// 	.smtp_utf8(true)
	// 	.connection_reuse(ConnectionReuseParameters::ReuseUnlimited)
}

fn init_simple_totp_long() -> TOTP {
	let TOTPSettings { secret, period } = &APP_SETTINGS.totp;
	let period = period.parse().expect("Could not parse TOTP period!");

	TOTPBuilder::new()
		.ascii_key(secret)
		.hash_function(Sha3_512)
		.period(period)
		.finalize()
		.expect("Could not initialize totp generator")
}

/// Creates a "keyed" totp, useful for generating user-based tokens
pub fn init_keyed_totp_long(key: &str) -> TOTP {
	let TOTPSettings { secret, period } = &APP_SETTINGS.totp;

	info!("init_keyed_totp_long: SECRET: {:?}, PERIOD: {:?}", secret, period);

	let period = period.parse().expect("Could not parse TOTP period!");

	TOTPBuilder::new()
		.ascii_key(&format!("{}_{}", key, secret))
		.hash_function(Sha3_512)
		.period(period)
		.finalize()
		.expect("Could not initialize totp generator")
}
