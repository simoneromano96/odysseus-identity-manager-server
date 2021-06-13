use crate::{
	auth::AuthErrors,
	user::{CreateUserInput, User, UserInfo},
};

use paperclip::actix::{
	api_v2_operation, post,
	web::{Data, Json},
};
use wither::mongodb::Database as MongoDatabase;

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
