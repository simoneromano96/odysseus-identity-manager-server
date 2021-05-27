use crate::{
	auth::AuthErrors,
	user::{User, UserInfo, UserInput},
	utils::verify_password,
};

use actix_session::Session;
use ory_hydra_client::{apis::admin_api, models::AcceptLoginRequest};
use paperclip::actix::{
	api_v2_operation, get, post,
	web::{Data, HttpResponse, Json},
};
use wither::mongodb::Database as MongoDatabase;

/// User signup
///
/// Creates a new user but doesn't log in the user
#[api_v2_operation]
#[post("/signup")]
pub async fn signup(db: Data<MongoDatabase>, new_user: Json<UserInput>) -> Result<Json<UserInfo>, AuthErrors> {
	// Create a user
	let user = User::create_user(&db, new_user.into_inner()).await?;
	Ok(Json(user.into()))
}

/// User login
///
/// Logs in the user via the provided credentials, will set a cookie session
#[api_v2_operation]
#[post("/login")]
pub async fn login(
	credentials: Json<UserInput>,
	session: Session,
	db: Data<MongoDatabase>,
) -> Result<Json<UserInfo>, AuthErrors> {
	match session.get("user_id")? {
		Some(user_id) => {
			// We can be sure that the user exists if there is a session, unless the cookie has been revoked
			let user = User::find_by_id(&db, &user_id)
				.await?
				.ok_or(AuthErrors::InvalidCookie)?;
			// Renew the session
			session.renew();
			// Give back user info
			Ok(Json(user.into()))
		}
		None => {
			// Find the user
			let user = User::find_by_username(&db, &credentials.username)
				.await?
				.ok_or(AuthErrors::UserNotFound)?;

			// Verify the password
			verify_password(&user.password, &credentials.password)?;

			// Create a session for the user
			session.set("user_id", user.id.clone().unwrap())?;

			// Give back user info
			Ok(Json(user.into()))
		}
	}
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
	session.get("user_id")?.ok_or(AuthErrors::UserNotLogged)?;
	session.clear();
	Ok(HttpResponse::Ok().body("Logged out"))
}
