use paperclip::actix::web::{scope, ServiceConfig};

use super::{
	get_consent, get_login, get_logout, local_login, post_consent, post_login, post_logout, signup, user_info,
	validate_email,
};

/// Configures all the auth routes
pub fn init_routes(cfg: &mut ServiceConfig) {
	// Local routes
	cfg.service(
		scope("/local")
			.service(signup)
			.service(validate_email)
			.service(local_login)
			.service(user_info),
	);

	// Oauth routes
	cfg.service(
		scope("/oauth")
			.service(get_consent)
			.service(get_login)
			.service(get_logout),
	);
	// Oauth routes
	// For some reason these must be separated
	cfg.service(
		scope("/oauth")
			.service(post_consent)
			.service(post_login)
			.service(post_logout),
	);
}
