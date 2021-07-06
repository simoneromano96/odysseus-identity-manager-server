use actix_cors::Cors;
use actix_redis::RedisSession;
use actix_web::{self, cookie, middleware, App, HttpServer};
use paperclip::{
	actix::{web::{scope}, OpenApiExt},
	v2::models::{Contact, DefaultApiRaw, Info, License},
};

use crate::{
	auth::{
		get_consent, get_login, get_logout, local_login, post_consent, post_login, post_logout, signup,
		user_info, validate_email,
	},
	settings::APP_SETTINGS,
	utils::{init_database, init_logger},
};

mod auth;
mod settings;
mod user;
mod utils;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const LICENSE: &str = env!("CARGO_PKG_LICENSE");
const LICENCE_URI: &str = env!("CARGO_PKG_LICENSE_FILE");
const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");

#[actix_web::main]
async fn main() -> std::io::Result<()> {
	init_logger();

	// Connect & sync indexes.
	let identity_database = init_database().await;

	HttpServer::new(move || {
		let cors = Cors::default()
			.allow_any_method()
			.allow_any_header()
			.expose_any_header()
			.supports_credentials()
			.allow_any_origin()
			.max_age(24 * 60 * 60);
		// .allowed_origin(&APP_SETTINGS.server.clienturi)
		let spec = create_base_spec();

		App::new()
			// enable logger
			.wrap(middleware::Logger::default())
			// cookie session middleware
			.wrap(
				RedisSession::new(&APP_SETTINGS.redis.uri, APP_SETTINGS.session.secret.as_bytes())
					// Don't allow the cookie to be accessed from javascript
					.cookie_http_only(true)
					// allow the cookie only from the current domain or with safe methods
					.cookie_same_site(cookie::SameSite::Lax),
			)
			.wrap(cors)
			.data(identity_database.clone())
			// Record services and routes from this line.
			.wrap_api_with_spec(spec)
			.service(
				scope("/api").service(
					scope("/v1")
						.service(
							scope("/local")
								.service(signup)
								.service(validate_email)
								.service(local_login)
								.service(user_info),
						)
						.service(
							scope("/oauth")
								.service(get_consent)
								.service(get_login)
								.service(get_logout)
								.service(post_consent)
								.service(post_login)
								.service(post_logout),
						),
				),
			)
			// Mount the JSON spec at this path.
			.with_json_spec_at("/openapi/docs")
			// Build the app
			.build()
	})
	.bind(format!("0.0.0.0:{:?}", APP_SETTINGS.server.port))?
	.run()
	.await
}

fn create_base_spec() -> DefaultApiRaw {
	let mut spec = DefaultApiRaw::default();

	// Add contact info
	let contact = Contact {
		name: Some(String::from("Simone Romano")),
		email: Some(String::from("simoneromano@tutanota.de")),
		url: Some(String::from("https://github.com/simoneromano96")),
	};

	// Add licence
	let license = License {
		name: Some(LICENSE.into()),
		url: Some(LICENCE_URI.into()),
	};

	// Create base specification
	spec.info = Info {
		version: VERSION.into(),
		title: "Odysseus identity manager server".into(),
		description: Some(DESCRIPTION.into()),
		contact: Some(contact),
		license: Some(license),
	};

	spec
}
