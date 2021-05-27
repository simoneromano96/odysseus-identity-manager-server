use crate::{
	models::{User, UserErrors, UserInfo, UserInput},
	settings::{APP_SETTINGS, ORY_HYDRA_CONFIGURATION},
	utils::{hash_password, verify_password, PasswordErrors},
};

use actix_session::Session;
use actix_web::{http::StatusCode, Error as ActixError, ResponseError};
use ory_hydra_client::{apis::admin_api, models::AcceptLoginRequest};
use paperclip::actix::{
	api_v2_operation, get, post,
	web::{Data, HttpResponse, Json},
};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use wither::{
	bson::{doc, oid::ObjectId},
	mongodb::Database as MongoDatabase,
	prelude::*,
	WitherError,
};

#[derive(Debug, Deserialize, Serialize)]
struct ErrorResponse {
	error: String,
}

#[derive(Error, Debug)]
pub enum AuthErrors {
	#[error("Internal server error")]
	ActixError(#[from] ActixError),
	#[error("Internal server error")]
	DatabaseError(#[from] WitherError),
	#[error("Could not create user")]
	UserCreationError(#[from] UserErrors),
	#[error("{0}")]
	PasswordError(#[from] PasswordErrors),
	#[error("Invalid cookie")]
	InvalidCookie,
	#[error("User not found")]
	UserNotFound,
	#[error("User is not logged in")]
	UserNotLogged,
}

impl ResponseError for AuthErrors {
	fn error_response(&self) -> HttpResponse {
		let error_response = ErrorResponse {
			error: self.to_string(),
		};
		HttpResponse::build(self.status_code()).json(error_response)
	}

	fn status_code(&self) -> StatusCode {
		match self {
			Self::UserCreationError(UserErrors::DatabaseError(_)) => StatusCode::BAD_REQUEST,
			Self::InvalidCookie => StatusCode::FORBIDDEN,
			Self::UserNotFound => StatusCode::NOT_FOUND,
			Self::PasswordError(_) => StatusCode::BAD_REQUEST,
			_ => StatusCode::INTERNAL_SERVER_ERROR,
		}
	}
}

/// User signup
///
/// Creates a new user but doesn't log in the user
///
/// Currently like this because of future developements
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
