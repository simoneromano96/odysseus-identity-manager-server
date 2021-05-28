use actix_cors::Cors;
use actix_redis::RedisSession;
use actix_web::{self, cookie, middleware, App, HttpServer};
use paperclip::actix::{web::scope, OpenApiExt};

use crate::{
	auth::{get_login, post_login, logout, signup, user_info},
	settings::APP_SETTINGS,
	utils::{init_database, init_logger},
};

mod auth;
mod settings;
mod user;
mod utils;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
	init_logger();

	// Connect & sync indexes.
	let identity_database = init_database().await;

	HttpServer::new(move || {
		App::new()
			// enable logger
			.wrap(middleware::Logger::default())
			// cookie session middleware
			.wrap(
				RedisSession::new(&APP_SETTINGS.redis.uri, APP_SETTINGS.session.secret.as_bytes())
					// Don't allow the cookie to be accessed from javascript
					.cookie_http_only(true)
					// allow the cookie only from the current domain
					.cookie_same_site(cookie::SameSite::Lax),
			)
			.wrap(Cors::permissive())
			.data(identity_database.clone())
			// Record services and routes from this line.
			.wrap_api()
			.service(
				scope("/api").service(
					scope("/v1")
						.service(signup)
						.service(get_login)
						.service(post_login)
						.service(user_info)
						.service(logout),
				),
			)
			// Mount the JSON spec at this path.
			.with_json_spec_at("/openapi")
			// Build the app
			.build()
	})
	.bind(format!("0.0.0.0:{:?}", APP_SETTINGS.server.port))?
	.run()
	.await
}
