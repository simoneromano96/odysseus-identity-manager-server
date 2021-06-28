use paperclip::actix::web::ServiceConfig;

use super::{get_consent, get_login, get_logout, post_consent, post_login, post_logout, signup, validate_email};

/// Configures all the auth routes
pub fn init_routes(cfg: &mut ServiceConfig) {
	// Local routes
	cfg.service(signup);
	cfg.service(validate_email);

	// Oauth routes
	cfg.service(get_consent);
	cfg.service(post_consent);
	cfg.service(get_login);
	cfg.service(post_login);
	cfg.service(get_logout);
	cfg.service(post_logout);
}
