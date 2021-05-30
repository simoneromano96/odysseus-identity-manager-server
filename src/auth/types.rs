use ory_hydra_client::models::CompletedRequest;
use paperclip::actix::Apiv2Schema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Apiv2Schema)]
pub struct AcceptedRequest {
	/// RedirectURL is the URL which you should redirect the user to once the authentication process is completed.
	pub redirect_to: String,
}

impl From<CompletedRequest> for AcceptedRequest {
	fn from(completed_request: CompletedRequest) -> Self {
		Self {
			redirect_to: completed_request.redirect_to,
		}
	}
}
