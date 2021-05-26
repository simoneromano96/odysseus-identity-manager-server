use std::env;

use config::{Config, Environment, File};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

pub static APP_SETTINGS: Lazy<Settings> = Lazy::new(Settings::init_config);

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Mongo database configuration
pub struct MongoConfig {
  /// DB Connection URI
  pub uri: String,
  /// DB Name
  pub database: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// App and server configuration
pub struct AppConfig {
  /// Server's port
  pub port: u16,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoggerConfig {
  /// What should the (terminal) logger print
  pub level: String,
  /// File logger path output
  pub path: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RedisConfig {
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionConfig {
    pub secret: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
  /// App and server configuration
  pub app: AppConfig,
  /// Logger configuration
  pub logger: LoggerConfig,
  /// Mongo database configuration
  pub mongo: MongoConfig,
  /// Redis configuration
  pub redis: RedisConfig,
  /// Session configuration
  pub session: SessionConfig,
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

    r
  }
}
