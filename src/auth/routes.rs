use crate::{
	auth::AuthErrors,
	settings::ORY_HYDRA_CONFIGURATION,
	user::{CreateUserInput, User, UserInfo},
	utils::verify_password,
};

use actix_session::Session;

use log::info;

use paperclip::actix::{
	api_v2_operation, get, post,
	web::{Data, HttpResponse, Json},
};
use wither::{bson::oid::ObjectId, mongodb::Database as MongoDatabase};

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
