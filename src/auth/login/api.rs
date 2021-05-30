use log::{error, info};
use ory_hydra_client::{
	apis::admin_api,
	models::{AcceptLoginRequest, CompletedRequest},
};

use crate::settings::ORY_HYDRA_CONFIGURATION;

use super::LoginErrors;

pub async fn handle_accept_login_request(
	subject: &str,
	login_challenge: &str,
) -> Result<CompletedRequest, LoginErrors> {
	info!("Accepting login request");

	let mut body = AcceptLoginRequest::new(subject.to_string());
	body.remember = Some(true);
	body.remember_for = Some(0);

	let accept_login_request = admin_api::accept_login_request(&ORY_HYDRA_CONFIGURATION, login_challenge, Some(body))
		.await
		.map_err(|e| {
			error!("{:?}", e);
			LoginErrors::HydraError
		})?;

	Ok(accept_login_request)
}
