use crate::{
	auth::AuthErrors,
	settings::ORY_HYDRA_CONFIGURATION,
	user::{CreateUserInput, User, UserInfo},
	utils::verify_password,
};

use actix_session::Session;
use actix_web::web::Query;
use log::info;
use ory_hydra_client::{apis::admin_api, models::AcceptLoginRequest};
use paperclip::actix::{
	api_v2_operation, get, post,
	web::{Data, HttpResponse, Json},
};
use wither::{bson::oid::ObjectId, mongodb::Database as MongoDatabase};

use super::{LoginInput, OauthLoginRequest};

/// User signup
///
/// Creates a new user but doesn't log in the user
#[api_v2_operation]
#[post("/signup")]
pub async fn signup(
	db: Data<MongoDatabase>,
	create_user_input: Json<CreateUserInput>,
) -> Result<Json<UserInfo>, AuthErrors> {
	// Create a user
	let user = User::create_user(&db, create_user_input.into_inner()).await?;
	Ok(Json(user.into()))
}

/// User login
///
/// Logs in the user via the provided credentials, will set a cookie session
#[api_v2_operation]
#[post("/login")]
pub async fn login(
	login_input: Json<LoginInput>,
	oauth_request: Query<OauthLoginRequest>,
	session: Session,
	db: Data<MongoDatabase>,
) -> Result<Json<UserInfo>, AuthErrors> {
	let user = match session.get("user_id")? {
		Some(user_id) => {
			// We can be sure that the user exists if there is a session, unless the cookie has been revoked
			let user = User::find_by_id(&db, &user_id)
				.await?
				.ok_or(AuthErrors::InvalidCookie)?;
			// Renew the session
			session.renew();

			user
		}
		None => {
			// Find the user
			let user = User::find_by_username(&db, &login_input.username)
				.await?
				.ok_or(AuthErrors::UserNotFound)?;

			// Verify the password
			verify_password(&user.password, &login_input.password)?;

			info!("User logged in: {:?}", &user);

			// Create a session for the user
			session.set("user_id", user.id.clone().unwrap())?;

			user
		}
	};

	if let Some(login_challenge) = oauth_request.into_inner().login_challenge {
		info!("Handling a login challenge");
		let body = Some(AcceptLoginRequest::new(user.id.clone().unwrap().to_string()));
		let login_request = admin_api::accept_login_request(&ORY_HYDRA_CONFIGURATION, &login_challenge, body)
			.await
			.map_err(|_e| AuthErrors::HydraError)?;
		
		info!("Hydra login response {:?}", login_request);
	}

	// Give back user info
	Ok(Json(user.into()))
}

/// User info
///
/// Gets the current user info if he is logged in
#[api_v2_operation]
#[get("/user-info")]
pub async fn user_info(session: Session, db: Data<MongoDatabase>) -> Result<Json<UserInfo>, AuthErrors> {
	// Get the session
	let id = session.get("user_id")?.ok_or(AuthErrors::UserNotLogged)?;

	// Search for the user
	let user = User::find_by_id(&db, &id).await?.ok_or_else(|| {
		// If we're in here the user sent a revoked cookie, delete it
		session.clear();
		AuthErrors::InvalidCookie
	})?;

	// Renew the session
	session.renew();
	// Send the user info
	Ok(Json(user.into()))
}

/// Logout
///
/// Logs out the current user invalidating the session if he is logged in
#[api_v2_operation]
#[get("/logout")]
pub async fn logout(session: Session) -> Result<HttpResponse, AuthErrors> {
	info!("Logout request");
	let _: ObjectId = session.get("user_id")?.ok_or(AuthErrors::UserNotLogged)?;
	info!("Got a logged user");
	session.clear();
	info!("Cleaned session");
	Ok(HttpResponse::Ok().body("Logged out"))
}
